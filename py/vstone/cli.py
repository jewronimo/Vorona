"""Command entry point; the run pipeline is delivered by task S7."""

import argparse


def main() -> None:
    """Display CLI availability without claiming the run pipeline exists."""
    parser = argparse.ArgumentParser(description="Vorona stone tile generator")
    parser.add_argument("--version", action="version", version="vstone 0.1.0")
    parser.parse_args()
    parser.print_help()
