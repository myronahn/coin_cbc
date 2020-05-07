pub mod raw;

pub use raw::Sense;

use std::collections::BTreeMap;
use std::os::raw::c_int;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Col(u32);
impl Col {
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Row(u32);
impl Row {
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Default, Clone)]
pub struct Model {
    num_cols: u32,
    num_rows: u32,
    col_lower: Vec<f64>,
    col_upper: Vec<f64>,
    row_lower: Vec<f64>,
    row_upper: Vec<f64>,
    obj_coefficients: Vec<f64>,
    weights: Vec<BTreeMap<Row, f64>>,
    is_integer: Vec<bool>,
    sense: Sense,
}

impl Model {
    pub fn add_col(&mut self) -> Col {
        let col = Col(self.num_cols);
        self.num_cols += 1;
        self.obj_coefficients.push(0.);
        self.weights.push(Default::default());
        self.is_integer.push(false);
        self.col_lower.push(0.);
        self.col_upper.push(f64::INFINITY);
        col
    }
    pub fn add_row(&mut self) -> Row {
        let row = Row(self.num_rows);
        self.num_rows += 1;
        self.row_lower.push(f64::NEG_INFINITY);
        self.row_upper.push(f64::INFINITY);
        row
    }
    pub fn set_weight(&mut self, row: Row, col: Col, weight: f64) {
        if weight == 0. {
            self.weights[col.as_usize()].remove(&row);
        } else {
            self.weights[col.as_usize()].insert(row, weight);
        }
    }
    pub fn set_integer(&mut self, col: Col) {
        self.is_integer[col.as_usize()] = true;
    }
    pub fn set_continuous(&mut self, col: Col) {
        self.is_integer[col.as_usize()] = false;
    }
    pub fn set_binary(&mut self, col: Col) {
        self.set_integer(col);
        self.set_col_lower(col, 0.);
        self.set_col_upper(col, 1.);
    }
    pub fn set_col_upper(&mut self, col: Col, value: f64) {
        self.col_upper[col.as_usize()] = value;
    }
    pub fn set_col_lower(&mut self, col: Col, value: f64) {
        self.col_lower[col.as_usize()] = value;
    }
    pub fn set_obj_coeff(&mut self, col: Col, value: f64) {
        self.obj_coefficients[col.as_usize()] = value;
    }
    pub fn set_row_upper(&mut self, row: Row, value: f64) {
        self.row_upper[row.as_usize()] = value;
    }
    pub fn set_row_lower(&mut self, row: Row, value: f64) {
        self.row_lower[row.as_usize()] = value;
    }
    pub fn set_obj_sense(&mut self, sense: Sense) {
        self.sense = sense;
    }
    pub fn to_raw(&self) -> raw::Model {
        let mut start = Vec::with_capacity(self.num_cols as usize + 1);
        let mut index = Vec::with_capacity(self.num_cols.max(self.num_rows) as usize);
        let mut value = Vec::with_capacity(self.num_cols.max(self.num_rows) as usize);
        start.push(0);
        for col_weights in &self.weights {
            for (r, w) in col_weights {
                index.push(r.0 as c_int);
                value.push(*w);
            }
            start.push(index.len() as c_int);
        }
        dbg!(&start);
        dbg!(&index);
        dbg!(&value);
        let mut raw = raw::Model::new();
        raw.load_problem(
            self.num_cols as usize,
            self.num_rows as usize,
            &start,
            &index,
            &value,
            Some(&self.col_lower),
            Some(&self.col_upper),
            Some(&self.obj_coefficients),
            Some(&self.row_lower),
            Some(&self.row_upper),
        );
        for (col, &is_int) in self.is_integer.iter().enumerate() {
            if is_int {
                raw.set_integer(col);
            } else {
                raw.set_continuous(col);
            }
        }
        raw.set_obj_sense(self.sense);
        raw
    }
    pub fn solve(&self) -> Solution {
        let mut raw = self.to_raw();
        raw.solve();
        Solution { raw }
    }
}

pub struct Solution {
    raw: raw::Model,
}
impl Solution {
    pub fn raw(&self) -> &raw::Model {
        &self.raw
    }
    pub fn into_raw(self) -> raw::Model {
        self.raw
    }
    pub fn col(&self, col: Col) -> f64 {
        self.raw.col_solution()[col.as_usize()]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn knapsack() {
        let mut m = Model::default();
        let row = m.add_row();
        m.set_row_upper(row, 10.);
        let cols = vec![
            m.add_col(),
            m.add_col(),
            m.add_col(),
            m.add_col(),
            m.add_col(),
        ];
        for &c in &cols {
            m.set_binary(c);
        }
        m.set_weight(row, cols[0], 2.);
        m.set_weight(row, cols[1], 8.);
        m.set_weight(row, cols[2], 4.);
        m.set_weight(row, cols[3], 2.);
        m.set_weight(row, cols[4], 5.);
        m.set_obj_coeff(cols[0], 5.);
        m.set_obj_coeff(cols[1], 3.);
        m.set_obj_coeff(cols[2], 2.);
        m.set_obj_coeff(cols[3], 7.);
        m.set_obj_coeff(cols[4], 4.);
        m.set_obj_sense(Sense::Maximize);

        let sol = m.solve();
        assert_eq!(raw::Status::Finished, sol.raw().status());
        assert_eq!(16., sol.raw().obj_value());
        assert_eq!(1., sol.col(cols[0]));
        assert_eq!(0., sol.col(cols[1]));
        assert_eq!(0., sol.col(cols[2]));
        assert_eq!(1., sol.col(cols[3]));
        assert_eq!(1., sol.col(cols[4]));
    }
}