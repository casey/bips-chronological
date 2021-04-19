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

    if readme.contains(&n) {
      let subdir = format!(
        "| [[https://github.com/bitcoin/bips/blob/master/bip-{:04}.mediawiki|{}]]\n",
        bip.n, bip.n
      );
      readme = readme.replace(&n, &format!("{}| {}\n", subdir, bip.date))
    }
  }

  readme = readme.replace("| 40", "| 40\n|");
  readme = readme.replace("| 41", "| 41\n|");
  readme = readme.replace("| 63", "| 63\n|");

  let rows_re = Regex::new(r"(?s)(\|-.*)\|}")?;

  let captures = if let Some(captures) = rows_re.captures(&readme) {
    captures
  } else {
    return Err(format!("Unable to extract table rows").into());
  };

  let table = &captures[1];

  let cells = table
    .lines()
    .filter(|line| line.starts_with("|"))
    .collect::<Vec<&str>>();

  let mut rows = Vec::new();

  for cell in cells {
    if cell.starts_with("|-") {
      rows.push(Vec::new());
    }
    rows.last_mut().unwrap().push(cell);
  }

  rows.sort_by_key(|row| row[2]);

  let mut replacement = String::new();

  for row in rows {
    for cell in row {
      replacement.push_str(cell);
      replacement.push('\n');
    }
  }

  replacement.push_str("\n|}");

  let readme = rows_re.replace(&readme, replacement);

  fs::write("README.mediawiki", &*readme)?;

  Ok(())
}
