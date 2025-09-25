use argh::FromArgs;
use indent::indent_all_by;
use petgraph::dot::{Config, Dot};
use petgraph::graph::Graph;
use phf::phf_map;
use regex::RegexSet;
use rpm::{FileMode, Package, PackageMetadata};
use std::collections::BTreeMap;

#[derive(FromArgs)]
/// Some RPM tools
struct Arrrpm {
    #[argh(subcommand)]
    subcommand: ArrrpmSubcommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum ArrrpmSubcommands {
    DepTree(SubcommandDepTree),
    Ls(SubcommandLs),
    Info(SubcommandInfo),
    Cat(SubcommandCat),
}

#[derive(FromArgs)]
/// Generate a dot graph with dependencies from a set of RPMs
#[argh(subcommand, name = "dep-tree")]
struct SubcommandDepTree {
    /// pattern for names to exclude
    #[argh(option, short = 'x')]
    exclude: Vec<String>,

    /// list of RPMs
    #[argh(positional)]
    rpms: Vec<String>,
}

#[derive(FromArgs)]
/// List files in the given RPM(s)
#[argh(subcommand, name = "ls")]
struct SubcommandLs {
    /// list of RPMs
    #[argh(positional)]
    rpms: Vec<String>,
}

#[derive(FromArgs)]
/// List info for the given RPM
#[argh(subcommand, name = "info")]
struct SubcommandInfo {
    /// RPM
    #[argh(positional)]
    rpm: String,
}

#[derive(FromArgs)]
/// Cat content from the RPM (mainly scriptlets)
#[argh(subcommand, name = "cat")]
struct SubcommandCat {
    /// RPM
    #[argh(positional)]
    rpm: String,

    /// scriptlets to output (default: all, can be: {{pre,post}}_{{install,uninstall,trans,untrans}})
    #[argh(option, short = 's')]
    scriptlets: Vec<String>,
}

type ScriptletMethod = fn(&PackageMetadata) -> Result<rpm::Scriptlet, rpm::Error>;
static SCRIPTLET_METHODS: phf::Map<&'static str, ScriptletMethod> = phf_map! {
    "pre_install" => PackageMetadata::get_pre_install_script,
    "post_install" => PackageMetadata::get_post_install_script,
    "pre_uninstall" => PackageMetadata::get_pre_uninstall_script,
    "post_uninstall" => PackageMetadata::get_post_uninstall_script,
    "pre_trans" => PackageMetadata::get_pre_trans_script,
    "post_trans" => PackageMetadata::get_post_trans_script,
    "pre_untrans" => PackageMetadata::get_pre_untrans_script,
    "post_untrans" => PackageMetadata::get_post_untrans_script,
};

fn main() {
    let arrrpm: Arrrpm = argh::from_env();

    match arrrpm.subcommand {
        ArrrpmSubcommands::DepTree(cmd) => {
            let excludes = match RegexSet::new(&cmd.exclude) {
                Ok(excludes) => excludes,
                Err(error) => {
                    panic!("Error in one of the exclude patterns: {:?}", error);
                }
            };

            let mut pkgs: BTreeMap<String, _> = BTreeMap::new();
            let mut tree: Graph<String, String, petgraph::Directed> = Graph::new();

            for rpm in cmd.rpms.iter() {
                let pkg = match PackageMetadata::open(rpm) {
                    Ok(pkg) => pkg,
                    Err(error) => {
                        eprintln!("Opening {:#?} failed: {:#?}, ignoring...", rpm, error);
                        continue;
                    }
                };

                let name = match pkg.get_name() {
                    Ok(name) => name.to_string(),
                    Err(error) => {
                        eprintln!(
                            "Reading name for {:#?} failed: {:#?}, ignoring...",
                            rpm, error
                        );
                        continue;
                    }
                };

                if excludes.is_match(&name) {
                    continue;
                }

                let reqs = match pkg.get_requires() {
                    Ok(reqs) => reqs,
                    Err(error) => {
                        eprintln!(
                            "Reading requirements for {:#?} failed: {:#?}, ignoring...",
                            rpm, error
                        );
                        continue;
                    }
                };
                let recs = match pkg.get_recommends() {
                    Ok(recs) => recs,
                    Err(error) => {
                        eprintln!(
                            "Reading recommends for {:#?} failed: {:#?}, ignoring...",
                            rpm, error
                        );
                        continue;
                    }
                };

                // add node for the package itself if it does not exist yet and get index for/from graph
                let parent_idx = *pkgs
                    .entry(name)
                    .or_insert_with_key(|k| tree.add_node(k.clone()));

                for dep in reqs.iter() {
                    if dep.name.contains('(') || dep.name.starts_with('/') {
                        // better to use the flags here?!
                        continue;
                    }
                    if excludes.is_match(&dep.name) {
                        continue;
                    }

                    let dep_idx = *pkgs
                        .entry(dep.name.clone())
                        .or_insert_with_key(|k| tree.add_node(k.clone()));
                    tree.add_edge(parent_idx, dep_idx, String::from("required"));
                }

                for dep in recs.iter() {
                    if dep.name.contains('(') || dep.name.starts_with('/') {
                        // better to use the flags here?!
                        continue;
                    }
                    if excludes.is_match(&dep.name) {
                        continue;
                    }

                    let dep_idx = *pkgs
                        .entry(dep.name.clone())
                        .or_insert_with_key(|k| tree.add_node(k.clone()));
                    tree.add_edge(parent_idx, dep_idx, String::from("recommended"));
                }
            }

            println!(
                "{:?}",
                Dot::with_attr_getters(
                    &tree,
                    &[Config::NodeNoLabel, Config::EdgeNoLabel],
                    &|_, er| format!("label = \"{}\"", er.weight()),
                    &|_, nr| format!("label = \"{}\"", nr.1),
                )
            );
        }

        ArrrpmSubcommands::Ls(cmd) => {
            for rpm in cmd.rpms.iter() {
                let pkg = match PackageMetadata::open(rpm) {
                    Ok(pkg) => pkg,
                    Err(error) => {
                        panic!("Opening {:#?} failed: {:#?}", rpm, error);
                    }
                };

                if cmd.rpms.len() > 1 {
                    println!("{}:", pkg.get_name().unwrap_or("INVALID"))
                }

                let fentries = match pkg.get_file_entries() {
                    Ok(fentries) => fentries,
                    Err(error) => {
                        panic!("Reading requirements for {:#?} failed: {:#?}", rpm, error);
                    }
                };

                let indent = if cmd.rpms.len() > 1 { "  - " } else { "" };

                for fentry in fentries.iter() {
                    match fentry.mode {
                        FileMode::Dir { permissions: _ } => {
                            println!("{}{}", indent, fentry.path.display());
                        }
                        FileMode::Regular { permissions: _ } => {
                            println!("{}{}", indent, fentry.path.display());
                        }
                        FileMode::SymbolicLink { permissions: _ } => {
                            println!("{}{} -> {}", indent, fentry.path.display(), fentry.linkto);
                        }
                        FileMode::Invalid {
                            raw_mode: _,
                            reason,
                        } => {
                            eprintln!("Invalid file entry for: {:?}: {}", fentry.path, reason);
                        }
                        _ => todo!(),
                    }
                }
            }
        }

        ArrrpmSubcommands::Info(cmd) => {
            let pkg = match PackageMetadata::open(&cmd.rpm) {
                Ok(pkg) => pkg,
                Err(error) => {
                    panic!("Opening {:#?} failed: {:#?}", cmd.rpm, error);
                }
            };

            println!("Name: {}", pkg.get_name().unwrap_or(""));
            println!("Epoch: {}", pkg.get_epoch().unwrap_or(0));
            println!("Version: {}", pkg.get_version().unwrap_or(""));
            println!("Release: {}", pkg.get_release().unwrap_or(""));
            println!("Arch: {}", pkg.get_arch().unwrap_or(""));
            println!("Summary: {}", pkg.get_summary().unwrap_or(""));
            println!("Vendor: {}", pkg.get_vendor().unwrap_or(""));
            println!("Packager: {}", pkg.get_packager().unwrap_or(""));
            println!("License: {}", pkg.get_license().unwrap_or(""));

            println!(
                "Description: |\n{}",
                indent_all_by(2, pkg.get_description().unwrap_or(""))
            );

            let mut available_scriptlets: Vec<String> = vec![];
            for (scriptlet_name, func) in &SCRIPTLET_METHODS {
                if func(&pkg).is_ok() {
                    available_scriptlets.push(scriptlet_name.to_string());
                }
            }
            println!("Scriptlets: [{}]", available_scriptlets.join(", "));

            println!("Requires:");
            if let Ok(reqs) = pkg.get_requires()  {
                for req in reqs {
                    if !req.version.is_empty() {
                        println!("  - {} {}", req.name, req.version)
                    } else {
                        println!("  - {}", req.name)
                    }
                }
            }

            println!("Recommends:");
            if let Ok(recs) = pkg.get_recommends()  {
                for rec in recs {
                    if !rec.version.is_empty() {
                        println!("  - {} ({})", rec.name, rec.version)
                    } else {
                        println!("  - {}", rec.name)
                    }
                }
            }
        }

        ArrrpmSubcommands::Cat(cmd) => {
            let pkg = match Package::open(&cmd.rpm) {
                Ok(pkg) => pkg,
                Err(error) => {
                    panic!("Opening {:#?} failed: {:#?}", cmd.rpm, error);
                }
            };

            for (scriptlet_name, func) in SCRIPTLET_METHODS.into_iter().filter(|(key, _)| {
                cmd.scriptlets.is_empty() || cmd.scriptlets.contains(&key.to_string())
            }) {
                if let Ok(scriptlet) = func(&pkg.metadata) {
                    println!(
                        "{}: |\n{}",
                        scriptlet_name,
                        indent_all_by(2, scriptlet.script)
                    );
                }
            }
        }
    }
}
