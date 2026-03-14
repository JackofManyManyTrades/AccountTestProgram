use std::collections::HashMap;
//Provides access to a file:
use std::fs::File;
//io is basic read/write and requires self, BufRead is a more efficient reading (buffered)
use std::io::{self, BufRead};
/*Makes a Path, which can be made from a String.  PathBuf is the owned version.  Not using PathBuf because a buffer will throttle the amount of info 
and PathBuf is mutable, but we know the Path and don't need to change it, so use Path.*/
use std::path::PathBuf;
//Walk directory
use csv::{ReaderBuilder, WriterBuilder, StringRecord};
use walkdir::WalkDir;
use std::error::Error;

// ======================================================
// DATA MODELS
// ======================================================

#[derive(Debug)]
struct Transaction {
    timestamp: String,
    account_id: String,
    owner_name: String,
    transaction_type: String,
    amount: f64,
}

#[derive(Debug)]
struct IngestionResult {
    balances: HashMap<String, f64>,
    owners: HashMap<String, String>,
    overdrawn: Vec<String>,
    errors: Vec<String>,
}

// ======================================================
// NORMALIZATION + VALIDATION HELPERS
// ======================================================

fn normalize_record(record: &StringRecord) -> Option<[String; 5]> {
    // Case 1: Proper CSV (already split)
    if record.len() == 5 {
        return Some([
            record[0].trim().to_string(),
            record[1].trim().to_string(),
            record[2].trim().to_string(),
            record[3].trim().to_string(),
            record[4].trim().to_string(),
        ]);
    }

    // Case 2: Entire line is quoted as ONE column
    if record.len() == 1 {
        let line = record[0].trim().trim_matches('"');
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() == 5 {
            return Some([
                parts[0].trim().to_string(),
                parts[1].trim().to_string(),
                parts[2].trim().to_string(),
                parts[3].trim().to_string(),
                parts[4].trim().to_string(),
            ]);
        }
    }

    None
}

fn parse_transaction(
    fields: [String; 5],
    line_number: usize,
) -> Result<Transaction, String> {
    let valid_types = ["deposit", "withdrawal"];

    if !valid_types.contains(&fields[3].as_str()) {
        return Err(format!(
            "Line {}: Invalid transaction type '{}'",
            line_number, fields[3]
        ));
    }

    let cleaned_amount = fields[4]
        .replace(',', "")
        .replace('$', "");

    let amount = cleaned_amount.parse::<f64>().map_err(|_| {
        format!("Line {}: Invalid amount '{}'", line_number, fields[4])
    })?;

    Ok(Transaction {
        timestamp: fields[0].clone(),
        account_id: fields[1].clone(),
        owner_name: fields[2].clone(),
        transaction_type: fields[3].clone(),
        amount,
    })
}

// ======================================================
// MAIN PROGRAM
// ======================================================

fn main() -> Result<(), Box<dyn Error>> {
    // ==================================================
    // FIND CSV FILE
    // ==================================================
    let csv_path: Option<PathBuf> = WalkDir::new("/*Please add the drive letter you wish to use, for example C:\\*/")
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .path()
                .file_name()
                .map_or(false, |name| name == "funidea.csv")
        })
        .map(|entry| entry.into_path());

    let csv_path = match csv_path {
        Some(p) => p,
        None => {
            eprintln!("Error: funidea.csv not found");
            return Ok(());
        }
    };

    println!("Found CSV at {}", csv_path.display());

    // ==================================================
    // OPEN CSV READER / CLEAN WRITER
    // ==================================================
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(&csv_path)?;

    let mut writer = WriterBuilder::new()
        .from_path("funidea_clean.csv")?;

    writer.write_record(&[
        "timestamp",
        "account_id",
        "owner_name",
        "transaction_type",
        "amount",
    ])?;

    // ==================================================
    // DATA STRUCTURES
    // ==================================================
    let mut balances: HashMap<String, f64> = HashMap::new();
    let mut owners: HashMap<String, String> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut overdrawn: Vec<String> = Vec::new();

    // ==================================================
    // PROCESS CSV
    // ==================================================
    for (i, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("Line {}: {}", i + 2, e));
                continue;
            }
        };

        let fields = match normalize_record(&record) {
            Some(f) => f,
            None => {
                errors.push(format!("Line {}: Malformed row", i + 2));
                continue;
            }
        };

        let tx = match parse_transaction(fields, i + 2) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        // ONE OWNER PER ACCOUNT
        match owners.get(&tx.account_id) {
            Some(existing) if existing != &tx.owner_name => {
                errors.push(format!(
                    "Line {}: Account {} has conflicting owners ('{}' vs '{}')",
                    i + 2,
                    tx.account_id,
                    existing,
                    tx.owner_name
                ));
                continue;
            }
            None => {
                owners.insert(tx.account_id.clone(), tx.owner_name.clone());
            }
            _ => {}
        }

        // UPDATE BALANCE
        let balance = balances.entry(tx.account_id.clone()).or_insert(0.0);

        match tx.transaction_type.as_str() {
            "deposit" => *balance += tx.amount,
            "withdrawal" => *balance -= tx.amount,
            _ => {}
        }

        if *balance < 0.0 {
            overdrawn.push(format!(
                "Line {}: Account {} overdrawn (${:.2})",
                i + 2,
                tx.account_id,
                balance
            ));
        }

        // WRITE CLEAN ROW
        writer.write_record(&[
            tx.timestamp,
            tx.account_id,
            tx.owner_name,
            tx.transaction_type,
            tx.amount.to_string(),
        ])?;
    }

    writer.flush()?;

    // ==================================================
    // FINAL OUTPUT
    // ==================================================
    println!("\n=== FINAL ACCOUNT SUMMARY ===");
    for (account, balance) in &balances {
        let owner = owners
    .get(account)
    .map(String::as_str)
    .unwrap_or("<unknown>");
        println!("Account {} ({}) -> ${:.2}", account, owner, balance);
    }

    if !overdrawn.is_empty() {
        println!("\n=== OVERDRAWN ACCOUNTS ===");
        for o in &overdrawn {
            println!("{}", o);
        }
    }

    if !errors.is_empty() {
        println!("\n=== ERRORS ===");
        for e in &errors {
            println!("{}", e);
        }
    }

    println!("\nClean CSV written to funidea_clean.csv");
    Ok(())
}


    // =========================================================
    // TODO 5: Read and process the CSV line by line
    // =========================================================
    //For loop declares line_number and line_result as variables in this function.
    //Ok(l) => means the line is formatted correctly, it only records errors and it does so in it's own array.'
    // =====================================================
    //parse is another way to do read, but reading a specific part of the statement.
    //v is an option under Ok, it means the field is valid._ for Err means invalid response.
