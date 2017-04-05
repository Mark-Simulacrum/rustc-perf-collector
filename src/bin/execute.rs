//! Execute benchmarks in a sysroot.

use std::str;
use std::path::{Path, PathBuf};
use std::process::Command;

use rustc_perf_collector::{Patch, Run};

use errors::{Result, ResultExt};
use sysroot::Sysroot;
use time_passes::{PassAverager, process_output};

pub struct Benchmark {
    pub name: String,
    pub path: PathBuf
}

impl Benchmark {
    pub fn command<P: AsRef<Path>>(&self, sysroot: &Sysroot, path: P) -> Command {
        let mut command = sysroot.command(path);
        command.current_dir(&self.path);
        command
    }

    /// Run a specific benchmark on a specific commit
    pub fn run(&self, sysroot: &Sysroot) -> Result<Vec<Patch>> {
        info!("processing {}", self.name);
        let output = self.command(sysroot, "make").arg("patches").output()?;
        let mut patches = str::from_utf8(&output.stdout)
            .chain_err(|| format!("make patches in {} returned non UTF-8 output", self.path.display()))?
            .split_whitespace()
            .collect::<Vec<_>>();
        if patches.is_empty() {
            patches.push("");
        }

        let mut patch_runs: Vec<Patch> = Vec::new();
        for _ in 0..3 {
            for patch in &patches {
                info!("running `make all{}`", patch);
                let output = self.command(sysroot, "make").arg(&format!("all{}", patch))
                    .env("CARGO_OPTS", "")
                    .env("CARGO_RUSTC_OPTS", "-Z time-passes")
                    .output()?;

                if !output.status.success() {
                    bail!("stderr non empty: {}", String::from_utf8_lossy(&output.stderr));
                }

                let patch_index = if let Some(p) = patch_runs.iter().position(|p_run| p_run.patch == *patch) {
                    p
                } else {
                    patch_runs.push(Patch {
                        patch: patch.to_string(),
                        name: self.name.clone(),
                        runs: Vec::new(),
                    });
                    patch_runs.len() - 1
                };

                let combined_name = format!("{}{}", self.name, patch);
                patch_runs[patch_index].runs.push(Run {
                    passes: process_output(&combined_name, output.stdout)?,
                    name: combined_name,
                });
            }
            if !self.command(sysroot, "make").arg("touch").status()?.success() {
                bail!("{}: make touch failed.", self.path.display());
            }
        }

        let mut patches = Vec::new();
        for patch_run in patch_runs {
            let mut runs = patch_run.runs.into_iter();
            let mut pa = PassAverager::new(runs.next().unwrap().passes);
            for run in runs {
                pa.average_with(run.passes)?;
            }
            patches.push(Patch {
                name: patch_run.name.clone(),
                patch: patch_run.patch.clone(),
                runs: vec![
                    Run {
                        name: patch_run.name + &patch_run.patch,
                        passes: pa.state,
                    }
                ],
            });
        }

        if !self.command(sysroot, "make").arg("clean").status()?.success() {
            bail!("{}: make touch failed.", self.path.display());
        }

        Ok(patches)
    }
}