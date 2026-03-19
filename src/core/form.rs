use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CaseId(u64);

trait Status {
    fn expected_type_tag() -> &'static str;
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
struct Review {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
struct Scoring {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
struct Completed {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
struct Cancelled {}

impl Status for Review {
    fn expected_type_tag() -> &'static str {
        "Review"
    }
}
impl Status for Scoring {
    fn expected_type_tag() -> &'static str {
        "Scoring"
    }
}
impl Status for Completed {
    fn expected_type_tag() -> &'static str {
        "Completed"
    }
}
impl Status for Cancelled {
    fn expected_type_tag() -> &'static str {
        "Cancelled"
    }
}

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

#[derive(Debug)]
struct FormError {
    details: String,
}

impl From<serde_json::Error> for FormError {
    fn from(error: serde_json::Error) -> Self {
        Self {
            details: format!("{}", error),
        }
    }
}

fn find<T>() -> Result<FormData<T>, FormError>
where
    T: Status + Deserialize<'static>,
{
    #[derive(Deserialize)]
    struct S {
        status: serde_json::Value,
    }

    let json = r#"{"case_id":1,"status":{"type":"Review"}}"#;
    let temp: S = serde_json::from_str(json)?;

    let received_type = temp.status.get("type")
        .and_then(|v| v.as_str())
        .expect("status must have a 'type' field");

    let expected_type = T::expected_type_tag();

    if received_type != expected_type {
        return Err(FormError {
            details: format!("unexpected type {}", received_type),
        });
    }
    let form = serde_json::from_str::<FormData<T>>(json)?;
    Ok(form)
}

#[cfg(test)]
mod tests {
    use super::{find, CaseId, Completed, ConnectionPool, Form, FormRepository, Review, SubmissionResult};

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

    #[test]
    fn test_deser() {
        let u = find::<Review>();
        println!("{:?}", u);
        let u = find::<Completed>();
    }
}
