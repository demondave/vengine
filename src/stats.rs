use std::{cmp::Ordering, collections::VecDeque};

use ahash::HashMap;

#[derive(Default)]
pub struct Stats {
    metrics: HashMap<String, Metric>,
}

struct Metric {
    name: String,
    unit: String,
    measurements: VecDeque<f64>,
    ranking: Ranking,
}

pub enum Ranking {
    Low,
    High,
}

impl Stats {
    pub fn add_metric(&mut self, key: String, name: String, unit: String, ranking: Ranking) {
        self.metrics.insert(
            key,
            Metric {
                name,
                unit,
                measurements: vec![0f64; 1000].into(),
                ranking,
            },
        );
    }

    pub fn push_metric(&mut self, key: &str, value: f64) {
        let metric = self.metrics.get_mut(key).unwrap();

        metric.measurements.pop_back();
        metric.measurements.push_front(value);
    }

    pub fn print(&mut self) {
        println!("                 | AVG (1000) | AVG (100) |  AVG (10) |    1% LOW |  0.1% LOW |");
        for metric in self.metrics.values_mut() {
            let mut values = metric.measurements.make_contiguous().to_vec();

            let avg_1000 = last_avg(&values, 1000);
            let avg_100 = last_avg(&values, 100);
            let avg_10 = last_avg(&values, 10);

            values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

            let (low_1, low_0_1) = match metric.ranking {
                Ranking::Low => (
                    n_percent_value(&values, 0.01),
                    n_percent_value(&values, 0.001),
                ),
                Ranking::High => (
                    n_percent_value(&values, 0.99),
                    n_percent_value(&values, 0.999),
                ),
            };

            println!(
                "{:<16} | {:>10.3} | {:>9.3} | {:>9.3} | {:>9.3} | {:>9.3} |",
                format!("{} [{}]", metric.name.clone(), metric.unit.clone()),
                avg_1000,
                avg_100,
                avg_10,
                low_1,
                low_0_1
            )
        }
        println!();
    }
}

fn n_percent_value(values: &[f64], p: f64) -> f64 {
    let index = (values.len() as f64 * p) as usize;

    values[index]
}

fn last_avg(values: &[f64], n: usize) -> f64 {
    let sum = values[0..n].iter().sum::<f64>();

    sum / (n as f64)
}
