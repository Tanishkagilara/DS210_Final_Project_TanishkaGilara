extern crate csv;
extern crate k_means;
extern crate chrono;
extern crate plotters;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use k_means::{KMeans, Point};
use chrono::{NaiveDateTime, Datelike};
use plotters::prelude::*;

#[derive(Debug, serde::Deserialize)]
struct CrimeRecord {
    ID: String,
    Case_Number: String,
    Date: NaiveDateTime,
    Block: String,
    IUCR: String,
    Primary_Type: String,
    Description: String,
    Location_Description: String,
    Arrest: bool,
    Domestic: bool,
    Beat: String,
    District: String,
    Ward: String,
    Community_Area: String,
    FBI_Code: String,
    X_Coordinate: Option<f64>,
    Y_Coordinate: Option<f64>,
    Year: i32,
    Updated_On: String,
    Latitude: Option<f64>,
    Longitude: Option<f64>,
    Location: String,
}

fn read_data(file_path: &str) -> Result<Vec<CrimeRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let mut record: CrimeRecord = result?;

        // Cleaning and transforming data
        record.Arrest = record.Arrest == "TRUE"; // Converting "TRUE" to true
        record.Domestic = record.Domestic == "TRUE"; // Converting "TRUE" to true
        record.Date = NaiveDateTime::parse_from_str(&record.Date, "%m/%d/%y %H:%M")?; // Parsing date string
        record.Year = record.Date.year(); // Extracting year from date

        // Converting empty strings to None for numeric fields
        record.X_Coordinate = if record.X_Coordinate.is_empty() {
            None
        } else {
            Some(record.X_Coordinate.parse().unwrap())
        };
        record.Y_Coordinate = if record.Y_Coordinate.is_empty() {
            None
        } else {
            Some(record.Y_Coordinate.parse().unwrap())
        };
        record.Latitude = if record.Latitude.is_empty() {
            None
        } else {
            Some(record.Latitude.parse().unwrap())
        };
        record.Longitude = if record.Longitude.is_empty() {
            None
        } else {
            Some(record.Longitude.parse().unwrap())
        };

        records.push(record);
    }

    Ok(records)
}

fn build_adjacency_list(records: &[CrimeRecord]) -> HashMap<String, HashSet<String>> {
    let mut adjacency_list: HashMap<String, HashSet<String>> = HashMap::new();

    for record in records {
        let incident_node = &record.ID;
        let related_nodes: HashSet<_> = records
            .iter()
            .filter(|&r| r.ID != *incident_node) // Excluding the incident node itself
            .filter(|&r| r.Date.date() == record.Date.date()) // 
            .map(|r| r.ID.clone())
            .collect();

        adjacency_list.insert(incident_node.clone(), related_nodes);
    }

    adjacency_list
}

fn six_degrees_of_distribution(
    adjacency_list: &HashMap<String, HashSet<String>>,
    start_node: &str,
) -> HashSet<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = Vec::new();

    queue.push(start_node.to_owned());
    visited.insert(start_node.to_owned());

    while !queue.is_empty() {
        let current_node = queue.remove(0);

        if let Some(neighbors) = adjacency_list.get(&current_node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push(neighbor.clone());
                }
            }
        }
    }

    visited
}

fn plot_temporal_trends(records: &[CrimeRecord]) {
    let mut date_counts: HashMap<NaiveDateTime, usize> = HashMap::new();

    for record in records {
        let date = record.Date.date();
        date_counts.entry(date).and_modify(|c| *c += 1).or_insert(1);
    }

    let mut dates: Vec<_> = date_counts.keys().cloned().collect();
    dates.sort(); // Sorting dates chronologically

    println!("Temporal Trends:");
    for date in &dates {
        println!("Date: {}, Count: {}", date, date_counts[date]);
    }

    // Plotting a histogram of temporal trends
    let root_area = BitMapBackend::new("temporal_trends.png", (800, 600)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root_area)
        .caption("Temporal Trends", ("sans-serif", 20).into_font())
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .build_cartesian_2d(dates[0]..dates.last().unwrap().succ(), 0..date_counts.values().max().unwrap() + 1)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Date")
        .y_desc("Count")
        .draw()
        .unwrap();

    chart
        .draw_series(dates.iter().zip(date_counts.values()).map(|(date, count)| {
            Circle::new(
                (date, *count as i32),
                3,
                (&BLACK).filled(),
            )
        }))
        .unwrap();
}

fn main() {
    let file_path = "chicago_crimes_sample_1.csv";
    match read_data(file_path) {
        Ok(records) => {
            // Filtering out records with missing coordinates
            let valid_records: Vec<_> = records
                .into_iter()
                .filter(|r| r.X_Coordinate.is_some() && r.Y_Coordinate.is_some())
                .collect();

            // Preparing points for clustering
            let points: Vec<Point<_>> = valid_records
                .iter()
                .map(|r| Point::new(vec![r.X_Coordinate.unwrap(), r.Y_Coordinate.unwrap()]))
                .collect();

            // Performing K-means clustering with k=3
            let k = 3;
            let kmeans = KMeans::new(&points, k);
            let clusters = kmeans.fit();

            // Print cluster centers and members
            for (idx, cluster) in clusters.iter().enumerate() {
                println!("Cluster {} Center: {:?}", idx, cluster.center());
                println!("Cluster {} Members:", idx);
                for member in cluster.points() {
                    let record_idx = points.iter().position(|p| p == member).unwrap(); // Find the original record index
                    println!("{:?}", valid_records[record_idx]);
                }
                println!();
            }

            // Performing temporal trend analysis
            plot_temporal_trends(&valid_records);

            // Building adjacency list and analyzing six degrees of distribution
            let adjacency_list = build_adjacency_list(&valid_records);
            let start_node = valid_records[0].ID.clone(); // Choosing the first record as the starting node
            let related_nodes = six_degrees_of_distribution(&adjacency_list, &start_node);
            println!("Six Degrees of Distribution (starting from {})", start_node);
            
            
         #[cfg(test)]
            mod tests {
                use super::*;
                use chrono::NaiveDate;
                use std::fs::{self, File};
                use std::path::Path;
            
                fn create_sample_record() -> CrimeRecord {
                    CrimeRecord {
                        ID: "1".to_string(),
                        Case_Number: "H123".to_string(),
                        Date: NaiveDateTime::new(NaiveDate::from_ymd(2020, 1, 1), chrono::NaiveTime::from_hms(12, 0, 0)),
                        Block: "100 XX BLOCK".to_string(),
                        IUCR: "0510".to_string(),
                        Primary_Type: "ASSAULT".to_string(),
                        Description: "AGGRAVATED: HANDGUN".to_string(),
                        Location_Description: "STREET".to_string(),
                        Arrest: true,
                        Domestic: false,
                        Beat: "123".to_string(),
                        District: "10".to_string(),
                        Ward: "1".to_string(),
                        Community_Area: "32".to_string(),
                        FBI_Code: "04A".to_string(),
                        X_Coordinate: Some(1155643.0),
                        Y_Coordinate: Some(1924568.0),
                        Year: 2020,
                        Updated_On: "01/01/2021".to_string(),
                        Latitude: Some(41.891398861),
                        Longitude: Some(-87.744384567),
                        Location: "41.891398861, -87.744384567".to_string(),
                    }
                }
            
                #[test]
                fn test_plot_temporal_trends() {
                    let records = vec![create_sample_record()];
                    let plot_path = "temporal_trends.png";
            
                    // Removing the file if it already exists to start with a clean state
                    let _ = fs::remove_file(plot_path);
            
                    // Generating the plot
                    plot_temporal_trends(&records);
            
                    // Checking if the file has been created
                    assert!(Path::new(plot_path).exists(), "The plot file was not created");
            
                    // Optionally, checking the file size to make sure it's not empty
                    let metadata = fs::metadata(plot_path).expect("Failed to retrieve file metadata");
                    assert!(metadata.len() > 0, "The plot file is empty");
            
                    // Cleaning up by removing the file after testing
                    let _ = fs::remove_file(plot_path);
                }
            }
            