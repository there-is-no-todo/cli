use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct Plan {
    id: Option<i32>,
    title: String,
    from_hr: Option<i32>,
    from_min: Option<i32>,
    to_hr: Option<i32>,
    to_min: Option<i32>,
}

// Impl trait for `Plan` that converts it to string
impl ToString for Plan {
    fn to_string(&self) -> String {
        let mut s = String::new();
        if let Some(id) = self.id {
            s.push_str(&format!("[{}] ", id));
        }
        if let Some(hr) = self.from_hr {
            s.push_str(&format!("{:02}", hr));
        }
        s.push_str(":");
        if let Some(min) = self.from_min {
            s.push_str(&format!("{:02}", min));
        }
        s.push_str(" - ");
        if let Some(hr) = self.to_hr {
            s.push_str(&format!("{:02}", hr));
        }
        s.push_str(":");
        if let Some(min) = self.to_min {
            s.push_str(&format!("{:02}", min));
        }
        s.push_str(" ");
        s.push_str(&self.title);
        return s;
    }
}

const URL: &str = "http://127.0.0.1:8000/";

/// Parse `hh:mm` into hour and minute.
/// If `hh:mm` is invalid, return `None`.
fn hm(time_str: String) -> Option<(i32, i32)> {
    let time: Vec<&str> = time_str.split(':').collect();
    if time.len() != 2 {
        return None;
    }
    let hr = time[0].parse::<i32>();
    let min = time[1].parse::<i32>();
    if let (Ok(hr), Ok(min)) = (hr, min) {
        // Prevent hr and min from being out of bound
        if hr < 0 || hr > 23 || min < 0 || min > 59 {
            return None;
        }
        return Some((hr, min));
    }
    return None;
}

/// Construct a `Plan` from `args`.
fn parse_args(args: Vec<String>) -> Option<Plan> {
    // Case 1, args is [todo, hh:mm, title]
    if args.len() == 3 {
        if let Some(to) = hm(args[1].clone()) {
            // Apply time to `to_hr` and `to_min`
            return Some(Plan {
                id: None,
                title: args[2].clone(),
                from_hr: None,
                from_min: None,
                to_hr: Some(to.0),
                to_min: Some(to.1),
            });
        }
    }
    // Case 2, args is [todo, hh:mm, ., title]
    if args.len() == 4 && args[2] == "." {
        if let Some(from) = hm(args[1].clone()) {
            // Apply time to `from_hr` and `from_min`
            return Some(Plan {
                id: None,
                title: args[3].clone(),
                from_hr: Some(from.0),
                from_min: Some(from.1),
                to_hr: None,
                to_min: None,
            });
        }
    }
    // Case 3, args is [todo, hh:mm, hh:mm, title]
    if args.len() == 4 && args[2] != "." {
        if let (Some(from), Some(to)) = (hm(args[1].clone()), hm(args[2].clone())) {
            // Apply time to `from_hr`, `from_min`, `to_hr`, `to_min`
            return Some(Plan {
                id: None,
                title: args[3].clone(),
                from_hr: Some(from.0),
                from_min: Some(from.1),
                to_hr: Some(to.0),
                to_min: Some(to.1),
            });
        }
    }

    // Case doesn't match
    return None;
}

/// Given a `Plan`, post it to the server as a json.
fn post_plan(plan: Plan) {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(URL)
        .json(&plan)
        .send()
        .expect("Failed to post plan");
    println!("Status: {}", res.status());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // Case 1, second argument is `--help` or `-h`, or no argument
    if args.len() == 1 || args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        println!("Usage: todo [hh:mm] [hh:mm] title");
        println!("            [hh:mm] . title");
        println!("            [hh:mm] title");
        println!("            -h --help");
        println!("            -v --version");
        println!("            -l --list");
        println!("            -g --get");
        println!("            -d --delete id");
        println!("            -c --clear");
        return;
    }
    // Case 2, second argument is `--version` or `-v`
    if args.len() == 2 && (args[1] == "--version" || args[1] == "-v") {
        println!("Version: 0.1.0");
        return;
    }
    // Case 3, second argument is `--list` or `-l`
    if args.len() == 2 && (args[1] == "--list" || args[1] == "-l") {
        let client = reqwest::blocking::Client::new();
        let res = client.get(URL).send().expect("Failed to get plans");
        println!("Status: {}", res.status());
        let ids: Vec<i32> = res.json().expect("Failed to parse json");
        for id in ids {
            // Get each plan
            let res = client
                .get(&format!("{}{}", URL, id))
                .send()
                .expect("Failed to get plan");
            let plan: Plan = res.json().expect("Failed to parse json");
            println!("{:?}", plan.to_string());
        }
        return;
    }
    // Case 4, second argument is `--delete` or `-d`
    if args.len() == 3 && (args[1] == "--delete" || args[1] == "-d") {
        let client = reqwest::blocking::Client::new();
        let res = client
            .delete(&format!("{}{}", URL, args[2]))
            .send()
            .expect("Failed to delete plan");
        println!("Status: {}", res.status());
        return;
    }
    // Case 5, second argument is `--get` or `-g`
    if args.len() == 3 && (args[1] == "--get" || args[1] == "-g") {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(&format!("{}{}", URL, args[2]))
            .send()
            .expect("Failed to get plan");
        println!("Status: {}", res.status());
        let plan: Plan = res.json().expect("Failed to parse json");
        println!("{:?}", plan.to_string());
        return;
    }
    // Case 6, second argument is `--clear` or `-c`
    if args.len() == 2 && (args[1] == "--clear" || args[1] == "-c") {
        let client = reqwest::blocking::Client::new();
        let res = client.delete(URL).send().expect("Failed to clear plans");
        println!("Status: {}", res.status());
        return;
    }
    // Other cases
    let plan = parse_args(args);
    if let Some(plan) = plan {
        post_plan(plan);
    } else {
        println!("Invalid arguments");
    }
}
