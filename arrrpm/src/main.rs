use argh::FromArgs;
use std::collections::BTreeMap;
use petgraph::graph::Graph;
use petgraph::dot::{Dot, Config};
use rpm::{PackageMetadata, FileMode};
use regex::RegexSet;

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
/// List files in the given RPM
#[argh(subcommand, name = "ls")]
struct SubcommandLs {
    /// RPM
    #[argh(positional)]
    rpm: String,
}

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
                let pkg = match PackageMetadata::open(&rpm) {
                    Ok(pkg) => pkg,
                    Err(error) => {
                        eprintln!("Opening {:#?} failed: {:#?}, ignoring...", rpm, error);
                        continue;
                    }
                };

                let name = match pkg.get_name() {
                    Ok(name) => name.to_string(),
                    Err(error) => {
                        eprintln!("Reading name for {:#?} failed: {:#?}, ignoring...", rpm, error);
                        continue;
                    }
                };

                if excludes.is_match(&name) {
                    continue;
                }

                let reqs = match pkg.get_requires() {
                    Ok(reqs) => reqs,
                    Err(error) => {
                        eprintln!("Reading requirements for {:#?} failed: {:#?}, ignoring...", rpm, error);
                        continue;
                    }
                };
                let recs = match pkg.get_recommends() {
                    Ok(recs) => recs,
                    Err(error) => {
                        eprintln!("Reading requirements for {:#?} failed: {:#?}, ignoring...", rpm, error);
                        continue;
                    }
                };

                // add node for the package itself if it does not exist yet and get index for/from graph
                let parent_idx = *pkgs.entry(name).or_insert_with_key(|k| tree.add_node(k.clone()));

                for dep in reqs.iter() {
                    if dep.name.contains("(") || dep.name.starts_with("/") { // better to use the flags here?!
                        continue;
                    }
                    if excludes.is_match(&dep.name) {
                        continue;
                    }

                    let dep_idx = *pkgs.entry(dep.name.clone()).or_insert_with_key(|k| tree.add_node(k.clone()));
                    tree.add_edge(parent_idx, dep_idx, String::from("required"));
                }

                for dep in recs.iter() {
                    if dep.name.contains("(") || dep.name.starts_with("/") { // better to use the flags here?!
                        continue;
                    }
                    if excludes.is_match(&dep.name) {
                        continue;
                    }

                    let dep_idx = *pkgs.entry(dep.name.clone()).or_insert_with_key(|k| tree.add_node(k.clone()));
                    tree.add_edge(parent_idx, dep_idx, String::from("recommended"));
                }
            }

            println!("{:?}", Dot::with_attr_getters(&tree,
                                                    &[Config::NodeNoLabel, Config::EdgeNoLabel],
                                                    &|_, er| format!("label = \"{}\"", er.weight()),
                                                    &|_, nr| format!("label = \"{}\"", nr.1),
                                                    ));
        },

        ArrrpmSubcommands::Ls(cmd) => {
            let pkg = match PackageMetadata::open(&cmd.rpm) {
                Ok(pkg) => pkg,
                Err(error) => {
                    panic!("Opening {:#?} failed: {:#?}", cmd.rpm, error);
                }
            };

            let fentries = match pkg.get_file_entries() {
                Ok(fentries) => fentries,
                Err(error) => {
                    panic!("Reading requirements for {:#?} failed: {:#?}", cmd.rpm, error);
                }
            };
            
            for fentry in fentries.iter() {
                match fentry.mode {
                    FileMode::Dir { permissions: _ } => {
                        println!("{}", fentry.path.display());
                    },
                    FileMode::Regular { permissions: _ } => {
                        println!("{}", fentry.path.display());
                    },
                    FileMode::SymbolicLink { permissions: _ } => {
                        println!("{} -> {}", fentry.path.display(), fentry.linkto);
                    }
                    FileMode::Invalid { raw_mode: _, reason } => {
                        eprintln!("Invalid file entry for: {:?}: {}", fentry.path, reason);
                    }
                    _ => todo!(),
                }
            }
        }
    }
}
