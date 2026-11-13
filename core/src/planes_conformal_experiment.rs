//! Diagnostic-only reference penalty reproduction; production solver is unchanged.
use super::*;

/// Experimental Claude penalty reproduction (inches), not production acceptance.
/// Freezes every fitted whole tile; returns full native reports without filtering failures.
pub fn experimental_conformal_penalty<S: SurfaceQueries>(
    surface: &S,
    t: &Topology,
    p: &Params,
    rim_y: Option<[f64; 2]>,
) -> Result<FitAndPolish, OptError> {
    let started = std::time::Instant::now();
    let mut problem = checked_native_problem(surface, t, p)?;
    let mut profile = FitProfile::default();
    let (x, _) = prototype_fit_profiled(&problem, prototype_start(&problem), &mut profile);
    let (_, baseline) = problem.evaluate(&x, 1.);
    let fit = with_distances(surface, baseline.clone());
    let fit_seconds = started.elapsed().as_secs_f64();
    problem.fixed_whole = Some(baseline.whole_tiles.clone());
    let residual = |x: &[f64]| {
        let (nn, c, _) = problem.geometry(x);
        let mut r: Vec<_> = c
            .iter()
            .zip(&problem.targets)
            .flat_map(|(&c, &t)| sub(c, t).map(|v| 0.05 * v))
            .collect();
        r.extend(x.iter().map(|v| 0.05 * v));
        for (i, fan) in problem.fans.iter().enumerate() {
            if !baseline.whole_tiles[i] {
                continue;
            }
            let area: f64 = (0..fan.len())
                .map(|k| {
                    dot(
                        cross(
                            sub(c[fan[k]], problem.points[i]),
                            sub(c[fan[(k + 1) % fan.len()]], problem.points[i]),
                        ),
                        nn[i].n,
                    )
                })
                .sum();
            let mut ids = fan.clone();
            if area < 0. {
                ids.reverse();
            }
            for k in 0..ids.len() {
                let prev = c[ids[(k + ids.len() - 1) % ids.len()]];
                let cur = c[ids[k]];
                if let Some([lo, hi]) = rim_y {
                    r.push(5. * (lo + 0.05 - cur[1]).max(0.));
                    r.push(5. * (cur[1] - hi + 0.05).max(0.));
                }
                let next = c[ids[(k + 1) % ids.len()]];
                let u = sub(cur, prev);
                let v = sub(next, cur);
                let l = norm(v);
                let angle = 180. - dot(cross(u, v), nn[i].n).atan2(dot(u, v)).to_degrees();
                r.push(5. * (p.min_edge + 0.2 - l).max(0.));
                r.push(5. * (l - p.max_edge + 0.2).max(0.));
                r.push(if angle <= 180. {
                    0.5 * (p.min_corner_angle_deg + 3. - angle).max(0.)
                } else {
                    0.
                });
                r.push(if angle > 180. {
                    0.5 * (angle - p.max_corner_angle_deg + 3.).max(0.)
                } else {
                    0.
                });
            }
        }
        r
    };
    let polish_start = std::time::Instant::now();
    let (post, it) = experimental_penalty_fit(x, residual);
    let (_, mut g) = problem.evaluate(&post, 1.);
    g.report.iterations = it;
    let g = with_distances(surface, g);
    let result = if g.report.violated.is_empty() {
        Ok(g)
    } else {
        Err(OptError {
            message: "experimental Claude penalty leaves native violations".into(),
            best: Some(Box::new(g)),
        })
    };
    Ok(FitAndPolish {
        fit,
        result,
        fit_seconds,
        polish_seconds: polish_start.elapsed().as_secs_f64(),
        total_seconds: started.elapsed().as_secs_f64(),
        fit_profile: profile,
    })
}

fn experimental_penalty_fit(
    mut x: Vec<f64>,
    residual: impl Fn(&[f64]) -> Vec<f64>,
) -> (Vec<f64>, usize) {
    use nalgebra::{DMatrix, DVector};
    let n = x.len();
    let order: Vec<_> = (0..2)
        .flat_map(|axis| (0..n / 2).map(move |i| 2 * i + axis))
        .collect();
    let mut radius = dot_slice(&x, &x).sqrt();
    if radius == 0. {
        radius = 1.;
    }
    let mut scales = vec![0f64; n];
    let mut alpha = 0.;
    let mut nfev = 1;
    let mut r = residual(&x);
    loop {
        let cost = 0.5 * dot_slice(&r, &r);
        let mut jac = DMatrix::zeros(r.len(), n);
        for (col, &j) in order.iter().enumerate() {
            let mut trial = x.clone();
            let h = f64::EPSILON.sqrt() * if x[j] < 0. { -1. } else { 1. } * x[j].abs().max(1.);
            trial[j] += h;
            let dx = trial[j] - x[j];
            let rr = residual(&trial);
            for k in 0..r.len() {
                jac[(k, col)] = (rr[k] - r[k]) / dx;
            }
        }
        // scipy x_scale="jac": nondecreasing column norms.
        for (col, scale) in scales.iter_mut().enumerate() {
            *scale = scale.max(jac.column(col).norm());
            if *scale == 0. {
                *scale = 1.;
            }
            jac.column_mut(col).scale_mut(1. / *scale);
        }
        let rv = DVector::from_column_slice(&r);
        let gradient = jac.transpose() * &rv;
        if gradient.iter().map(|v| v.abs()).fold(0., f64::max) < 1e-8 || nfev == 60 {
            return (x, nfev);
        }
        // Preserve the original tall-SVD floating-point trajectory. QR reduction
        // is equivalent in exact arithmetic but perturbs TRF steps enough to
        // change the basin and frozen whole-tile domain on organic cylinders.
        // Keep single-fit reuse and exact colored polish probes independently.
        let svd = jac.clone().svd(true, true);
        let singular = svd.singular_values;
        let uf = svd.u.unwrap().transpose() * rv;
        let v = svd.v_t.unwrap().transpose();
        let suf = singular.component_mul(&uf);
        let full_rank = singular[n - 1] > f64::EPSILON * r.len() as f64 * singular[0];
        let mut stop = false;
        while nfev < 60 {
            let gn = -(&v * uf.component_div(&singular));
            let step = if full_rank && gn.norm() <= radius {
                alpha = 0.;
                gn
            } else {
                let phi = |a: f64| {
                    let denom = singular.map(|s| s * s + a);
                    let norm = suf.component_div(&denom).norm();
                    let derivative = -suf
                        .iter()
                        .zip(denom.iter())
                        .map(|(u, d)| u * u / (d * d * d))
                        .sum::<f64>()
                        / norm;
                    (norm - radius, derivative)
                };
                let mut upper = suf.norm() / radius;
                let mut lower = if full_rank {
                    let (f, d) = phi(0.);
                    -f / d
                } else {
                    0.
                };
                if !full_rank && alpha == 0. {
                    alpha = (0.001 * upper).max((lower * upper).sqrt());
                }
                for _ in 0..10 {
                    if alpha < lower || alpha > upper {
                        alpha = (0.001 * upper).max((lower * upper).sqrt());
                    }
                    let (f, d) = phi(alpha);
                    if f < 0. {
                        upper = alpha;
                    }
                    let ratio = f / d;
                    lower = lower.max(alpha - ratio);
                    alpha -= (f + radius) * ratio / radius;
                    if f.abs() < 0.01 * radius {
                        break;
                    }
                }
                let mut step = -(&v * suf.component_div(&singular.map(|s| s * s + alpha)));
                step *= radius / step.norm();
                step
            };
            let step_norm = step.norm();
            let predicted = -(0.5 * (&jac * &step).norm_squared() + gradient.dot(&step));
            let mut trial = x.clone();
            for (col, &j) in order.iter().enumerate() {
                trial[j] += step[col] / scales[col];
            }
            let rr = residual(&trial);
            nfev += 1;
            if rr.iter().any(|v| !v.is_finite()) {
                radius = 0.25 * step_norm;
                continue;
            }
            let reduction = cost - 0.5 * dot_slice(&rr, &rr);
            let ratio = if predicted > 0. {
                reduction / predicted
            } else if predicted == 0. && reduction == 0. {
                1.
            } else {
                0.
            };
            let next_radius = if ratio < 0.25 {
                0.25 * step_norm
            } else if ratio > 0.75 && step_norm > 0.95 * radius {
                2. * radius
            } else {
                radius
            };
            stop = (reduction < 1e-8 * cost && ratio > 0.25)
                || step_norm < 1e-8 * (1e-8 + dot_slice(&x, &x).sqrt());
            if !stop {
                alpha *= radius / next_radius;
                radius = next_radius;
            }
            if reduction > 0. {
                x = trial;
                r = rr;
                break;
            }
            if stop {
                break;
            }
        }
        if stop || nfev == 60 {
            return (x, nfev);
        }
    }
}
