use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CaseId(u64);

trait Status {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

struct Review {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Scoring {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Completed {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Cancelled {}

impl Status for Review {}
impl Status for Scoring {}
impl Status for Completed {}
impl Status for Cancelled {}

trait Form {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FormData<Status = Scoring> {
    case_id: CaseId,
    status: Status,
}

impl<T> Form for FormData<T>
where
    T: Serialize,
{
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

enum SubmissionResult {
    Escalation(FormData<Review>),
    Closed(FormData<Completed>),
}

impl FormData<Review> {
    fn review_question(&mut self) {}

    fn submit(self) -> SubmissionResult {
        if self.case_id == CaseId(0) {
            SubmissionResult::Closed(FormData {
                case_id: self.case_id,
                status: Completed {},
            })
        } else {
            SubmissionResult::Escalation(FormData {
                case_id: self.case_id,
                status: Review {},
            })
        }
    }
}

impl FormData<Scoring> {
    fn score(self) -> FormData<Review> {
        FormData {
            case_id: self.case_id,
            status: Review {},
        }
    }
}

trait FormRepository {
    fn find_review_case(&self, case_id: CaseId) -> Option<FormData<Review>>;
    fn find_score_case(&self, case_id: CaseId) -> Option<FormData<Scoring>>;

    fn save(&self, form: &impl Form);
}

struct ConnectionPool {}

impl FormRepository for ConnectionPool {
    fn find_review_case(&self, case_id: CaseId) -> Option<FormData<Review>> {
        let form = FormData {
            case_id,
            status: Review {},
        };
        Some(form)
    }

    fn find_score_case(&self, case_id: CaseId) -> Option<FormData<Scoring>> {
        let form = FormData {
            case_id,
            status: Scoring {},
        };
        Some(form)
    }

    fn save(&self, form: &impl Form) {
        let json = form.to_json().expect("could not serialize form");
        println!("{}", json);
    }
}

#[cfg(test)]
mod tests {
    use super::{CaseId, ConnectionPool, Form, FormRepository, SubmissionResult};
    use serde::Serialize;

    #[test]
    fn test_score() {
        let repository = ConnectionPool {};
        let scoring_form = repository
            .find_score_case(CaseId(1))
            .expect("could not find score");
        let review_form = scoring_form.score();
        repository.save(&review_form);
    }

    #[test]
    fn test() {
        let repository = ConnectionPool {};
        let mut review_form = repository
            .find_review_case(CaseId(2))
            .expect("could not find review");
        review_form.review_question();
        match review_form.submit() {
            SubmissionResult::Closed(f) => {
                println!("closed form");
                let s = f.to_json().expect("could not serialize form");
                println!("{}", s);
            }
            SubmissionResult::Escalation(escalation) => println!(
                "escalation form {}",
                escalation.to_json().expect("could not serialize form")
            ),
        }
    }
}
