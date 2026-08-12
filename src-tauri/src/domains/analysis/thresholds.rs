//! Thresholds still consumed by the current evidence pipeline.

pub mod kda {
    pub const S_PLUS_GRADE: f64 = 8.0;
    pub const S_GRADE: f64 = 6.0;
    pub const A_GRADE: f64 = 4.0;
    pub const B_GRADE: f64 = 2.5;
    pub const C_GRADE: f64 = 1.5;

    pub fn grade_from_kda(kda: f64) -> &'static str {
        if kda >= S_PLUS_GRADE {
            "S+"
        } else if kda >= S_GRADE {
            "S"
        } else if kda >= A_GRADE {
            "A"
        } else if kda >= B_GRADE {
            "B"
        } else if kda >= C_GRADE {
            "C"
        } else {
            "D"
        }
    }
}
