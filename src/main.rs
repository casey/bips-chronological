use std::{error::Error, fs};

use chrono::NaiveDate;
use regex::Regex;

struct Bip {
  n:    u64,
  date: NaiveDate,
}

fn main() -> Result<(), Box<dyn Error>> {
  let bip_re = Regex::new("^bip-([0-9]{4}).mediawiki$")?;
  let created_re = Regex::new(r"^\s*Created:\s*(.*)$")?;

  let mut bips = Vec::new();

  'outer: for result in fs::read_dir("bips")? {
    let entry = result?;

    let name = entry.file_name().to_string_lossy().to_string();

    let captures = if let Some(captures) = bip_re.captures(&name) {
      captures
    } else {
      continue;
    };

    let n = captures[1].parse::<u64>()?;

    let text = fs::read_to_string(entry.path())?;

    for line in text.lines() {
      if let Some(captures) = created_re.captures(line) {
        let date = NaiveDate::parse_from_str(&captures[1], "%Y-%m-%d")?;

        bips.push(Bip { n, date });

        continue 'outer;
      }
    }

    return Err(format!("Unable to parse BIP created date: {}", n).into());
  }

  let mut readme = fs::read_to_string("bips/README.mediawiki")?;

  readme = readme.replace("!Number", "!Number\n!Created");

  for bip in &bips {
    let n = format!("| [[bip-{:04}.mediawiki|{}]]\n", bip.n, bip.n);
    readme = readme.replace(&n, &format!("{}| {}\n", n, bip.date))
  }

  fs::write("README.mediawiki", &readme)?;

  bips.sort_by_key(|bip| bip.date);

  for Bip { n, date } in bips {
    println!("{:04} {}", n, date);
  }

  Ok(())
}
