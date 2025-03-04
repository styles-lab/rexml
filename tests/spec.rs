use std::{
    panic::catch_unwind,
    path::{Path, PathBuf},
};

use rexml::reader::lexer::XmLexer;

#[test]
fn test_specs() {
    // _ = pretty_env_logger::try_init();

    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");

    let mut xml_files = vec![];

    for entry in std::fs::read_dir(&root_dir).unwrap() {
        let entry = entry.unwrap();

        if entry.file_type().unwrap().is_dir() {
            continue;
        }

        if let Some(extension) = entry.path().extension() {
            if extension == "xml" || extension == "svg" {
                xml_files.push((
                    root_dir.join(entry.path()).join(entry.path()),
                    entry.file_name(),
                ));
            }
        }
    }

    let mut succ = 0;
    let mut faileds = 0;

    for (xml, file_name) in xml_files {
        print!("xml {:?} ... ", file_name);

        match catch_unwind(move || test_xml(xml)) {
            Ok(_) => {
                println!("ok");
                succ += 1;
            }
            Err(e) => {
                println!("failed");

                faileds += 1;

                eprintln!("{:?}", e);
            }
        }

        // test_svg(svg, output);
    }

    if faileds > 0 {
        panic!("spec result: ok {} passed; {} failed;", succ, faileds);
    } else {
        println!("spec result: ok {} passed; {} failed;", succ, faileds);
    }
}

fn test_xml(xml: impl AsRef<Path>) {
    let content = std::fs::read_to_string(xml).unwrap();

    let tokens = XmLexer::from(content)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    print!("{} ", tokens.len());
}
