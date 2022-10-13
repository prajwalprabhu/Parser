use chrono::offset;
use chrono::prelude::*;
use regex::Regex;
use std::format;
use std::fs;
use std::path::PathBuf;
use tauri::api::path::desktop_dir;

struct Library {
    start: u32,
    end: u32,
    openday: u32,
    lib_data: String,
}
pub fn run(price: f64) {
    let mut lib_folder = desktop_dir().unwrap();
    lib_folder.push("Parser");
    let mut lib_path = lib_folder.clone();
    lib_path.push("Library");
    let mut catalog_path = lib_folder.clone();
    catalog_path.push("catalog.xml");
    let library = fs::read_dir(format!("{}", lib_path.display()))
        .unwrap()
        .map(|entry| entry.unwrap().path());

    let mut file_content = String::from("<catalog>\n");
    let mut withzero: Vec<Library> = Vec::new();
    let mut minute: u32 = offset::Local::now().minute();
    let hour = offset::Local::now().hour();
    minute = if minute < 10 { minute * 10 } else { minute };
    let time = format!("{}{}", hour, minute).parse::<u32>().unwrap();
    let day = offset::Local::now().weekday().num_days_from_sunday() + 1;

    for entry in library {
        let lib_data = fs::read_to_string(entry).unwrap().to_string();

        let mut content = rgx_str(Regex::new(r".*library.*").unwrap(), lib_data.to_string());
        let mut rgx = Regex::new(r#"starttime=".*?"|endtime=".*?"|opendays=".*?""#).unwrap();
        content = rgx
            .find_iter(content.as_str())
            .map(|x| x.as_str().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        rgx = Regex::new(r#"\d+"#).unwrap();

        let digits = rgx
            .find_iter(content.as_str())
            .map(|x| x.as_str().to_string())
            .collect::<Vec<String>>();

        let start = digits[0].parse::<u32>().unwrap();
        let end = digits[1].parse::<u32>().unwrap();
        let openday = digits[2].parse::<u32>().unwrap();
        println!(
            "time:{} day:{} start:{} end:{} open:{}",
            time, day, start, end, openday
        );
        if openday == 0 {
            withzero.push(Library {
                start,
                end,
                openday,
                lib_data,
            });
            continue;
        }
        file_content += get_content(
            time,
            day,
            start,
            end,
            openday,
            lib_data,
            price,
            lib_folder.clone(),
        )
        .as_str();

        if file_content != "<catalog>\n" {
            break;
        }
    }

    if file_content == "<catalog>" {
        for v in withzero.iter() {
            file_content.push_str(
                format!(
                    "{}\n",
                    get_content(
                        time,
                        day,
                        v.start,
                        v.end,
                        v.openday,
                        v.lib_data.to_string(),
                        price,
                        lib_folder.clone()
                    )
                )
                .as_str(),
            );
            if file_content != "<catalog>" {
                break;
            }
        }
    }

    if file_content == "<catalog>" {
        file_content += "\n<empty>No library is open right now regarding your current time</empty>";
    }
    file_content += "\n</catalog>";

    fs::write(format!("{}", catalog_path.display()), file_content).unwrap();
}

fn get_content(
    time: u32,
    day: u32,
    start: u32,
    end: u32,
    openday: u32,
    lib_data: String,
    price: f64,
    lib_path: PathBuf,
) -> String {
    let mut file_path = lib_path.clone();
    file_path.push("files");
    let mut file_content: String = String::new();
    if (day == openday || openday == 0) && time > start && time < end {
        let rgx = Regex::new(r#"\s*<book.*>"#).unwrap();
        let mut ids = rgx
            .find_iter(lib_data.as_str())
            .map(|x| x.as_str().to_string())
            .collect::<Vec<String>>();

        ids = rgx_vec(Regex::new(r#"id=".*?""#).unwrap(), ids);
        ids = rgx_vec(Regex::new(r#"".*?""#).unwrap(), ids);
        ids = ids
            .iter()
            .map(|x| x.to_string().replace('\"', ""))
            .collect::<Vec<String>>();
        let mut total_price = 0.0;
        for id in ids {
            let mut file_path_xml = file_path.clone();
            file_path_xml.push(format!("{id}.xml"));
            let book_data = fs::read_to_string(format!("{}", file_path_xml.display()))
                .unwrap()
                .to_string();
            let _file_price = rgx_str(Regex::new(r".*price.*").unwrap(), book_data.to_string());
            let file_price = rgx_str(Regex::new(r"[\d\.]+").unwrap(), _file_price.to_string())
                .parse::<f32>()
                .unwrap();
            if total_price <= price {
                file_content.push_str(format!("{}\n", book_data).as_str());
                total_price += file_price as f64;
            } else {
                break;
            }
        }
    }
    file_content
}

fn rgx_vec(rgx: Regex, ids: Vec<String>) -> Vec<String> {
    return ids
        .iter()
        .map(|x| rgx_str(rgx.clone(), x.to_string()))
        .collect::<Vec<String>>();
}

fn rgx_str(rgx: Regex, str: String) -> String {
    return rgx
        .find_iter(str.as_str())
        .map(|x| x.as_str().to_string())
        .collect::<Vec<String>>()[0]
        .to_string();
}
