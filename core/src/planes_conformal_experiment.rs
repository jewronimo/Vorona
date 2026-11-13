//! Explicit unguarded diagnostic orchestration; production uses guarded reference polish.
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
    let polish_start = std::time::Instant::now();
    let (post, it) = reference_penalty_fit(x, |x| {
        let mut residual = reference_penalty_residual(&problem, x);
        // Explicit diagnostic-only rim experiment, not a production objective.
        if let Some([lo, hi]) = rim_y {
            let (_, corners, _) = problem.geometry(x);
            for (i, fan) in problem.fans.iter().enumerate() {
                if baseline.whole_tiles[i] {
                    for &k in fan {
                        residual.push(5. * (lo + 0.05 - corners[k][1]).max(0.));
                        residual.push(5. * (corners[k][1] - hi + 0.05).max(0.));
                    }
                }
            }
        }
        residual
    });
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
