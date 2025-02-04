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
}

impl Metric {
    fn get_n_percent_value(&mut self, p: f64) -> f64 {
        let index = (self.measurements.len() as f64 * p) as usize;
        let contiguous = self.measurements.make_contiguous();

        contiguous
            .select_nth_unstable_by(index, |a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        contiguous[index]
    }

    fn get_last_avg(&mut self, n: usize) -> f64 {
        let sum = self.measurements.make_contiguous()[0..n]
            .iter()
            .sum::<f64>();

        sum / (n as f64)
    }
}

impl Stats {
    pub fn add_metric(&mut self, key: String, name: String, unit: String) {
        self.metrics.insert(
            key,
            Metric {
                name,
                unit,
                measurements: vec![0f64; 1000].into(),
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
            println!(
                "{:<16} | {:>10.3} | {:>9.3} | {:>9.3} | {:>9.3} | {:>9.3} |",
                format!("{} [{}]", metric.name.clone(), metric.unit.clone()),
                metric.get_last_avg(1000),
                metric.get_last_avg(100),
                metric.get_last_avg(10),
                metric.get_n_percent_value(0.01),
                metric.get_n_percent_value(0.001)
            )
        }
    }
}
