use argh::FromArgs;

#[derive(FromArgs)]
/// Generate a dot graph with dependencies from a set of RPMs
struct RPMDeps {
    /// list of RPMs
    #[argh(positional)]
    rpms: Vec<String>,
}

fn main() {
    let rpmdeps: RPMDeps = argh::from_env();

    for rpm in rpmdeps.rpms.iter() {
        let pkg = match rpm::Package::open(rpm) {
            Ok(pkg) => pkg,
            Err(e) => {
                println!("Opening {:#?} failed: {:#?}, ignoring...", rpm, e);
                continue;
            }
        };
        let name = match pkg.metadata.get_name() {
            Ok(name) => name,
            Err(e) => {
                println!("Reading name from {:#?} failed: {:#?}, ignoring...", rpm, e);
                continue;
            }
        };
        let reqs = match pkg.metadata.get_requires() {
            Ok(reqs) => reqs,
            Err(e) => {
                println!("Reading requirements from {:#?} failed: {:#?}, ignoring...", rpm, e);
                continue;
            }
        };

        for dep in reqs.iter() {
            if dep.name.starts_with("/") || dep.name.contains("(") {
                continue;
            }
            println!("{:#?} -> {:#?}", name, dep.name);
        }
    }
}
