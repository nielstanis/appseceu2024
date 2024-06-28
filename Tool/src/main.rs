use std::{fs::File, io::BufReader, process::{Command, Stdio}};
use glob::glob;
use serde_json_path::JsonPath;
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Find all nested POM files Maven
    for entry in glob("./**/pom.xml").expect("Failed to read glob pattern") {
        match entry {
            Ok(mut path) => {
                //Get list of transitive dependencies using MVN tool
                //mvn dependency:tree -DoutputFile=deps.json -DoutputType=json
                println!("Found `{}`, parsing xml for getting transitive dependency tree", &path.display());
                Command::new("mvn")
                    .stdout(Stdio::null())
                    .arg(format!("-f{}", path.display()))
                    .args(&["dependency:tree","-DoutputFile=deps.json", "-DoutputType=json"])
                    .spawn()
                    .expect("mvn command failed to start");
                
                path.set_file_name("deps.json");
                let reader = BufReader::new(File::open(path)?);
                let json: serde_json::Value =
                serde_json::from_reader(reader).expect("JSON was not well-formatted");

                let path = JsonPath::parse("$.children[:]").unwrap();
                let nodes = path.query(&json).all();
                for child in nodes {
                    let group = child.as_object().unwrap();
                    let version = group.get("version").unwrap().as_str().unwrap();
                    let group_id = group.get("groupId").unwrap().as_str().unwrap();
                    let artifact_id = group.get("artifactId").unwrap().as_str().unwrap();
                    let url = format!("https://api.deps.dev/v3/systems/maven/packages/{}%3A{}/versions/{}", group_id, artifact_id, version );

                    let resp : serde_json::Value = reqwest::get(url).await?
                        .json()
                        .await?;

                    let source_path = JsonPath::parse("$.links[:]").unwrap();
                    let source_url= source_path.query(&resp).all();
                    println!("{:?}", source_url)

                    //Now with SOURCE_URL get scorecard
                    //e.g. https://api.scorecard.dev/projects/github.com/quartznet/quartznet
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    //Find all NPM packages.json
    for entry in glob("./**/packages.json").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("{:?}", path.display()),
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}