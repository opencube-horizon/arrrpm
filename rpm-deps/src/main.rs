use argh::FromArgs;
use std::collections::BTreeMap;
use petgraph::graph::Graph;
use petgraph::dot::{Dot, Config};
use rpm::PackageMetadata;
use regex::RegexSet;

#[derive(FromArgs)]
/// Generate a dot graph with dependencies from a set of RPMs
struct RPMDeps {
    /// pattern for names to exclude
    #[argh(option, short = 'x')]
    exclude: Vec<String>,

    /// list of RPMs
    #[argh(positional)]
    rpms: Vec<String>,
}

fn main() {
    let rpmdeps: RPMDeps = argh::from_env();
    let excludes = match RegexSet::new(&rpmdeps.exclude) {
        Ok(excludes) => excludes,
        Err(error) => {
            panic!("Error in one of the exclude patterns: {:?}", error);
        }
    };

    let mut pkgs: BTreeMap<String, _> = BTreeMap::new();
    let mut tree: Graph<String, String, petgraph::Directed> = Graph::new();

    for rpm in rpmdeps.rpms.iter() {
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
}
