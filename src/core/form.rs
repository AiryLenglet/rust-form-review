use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CaseId(u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Status {
    Review,
    Scoring,
    Completed,
    Cancelled,
}

trait Form {
    fn serialize(&self) -> Result<serde_json::Value, serde_json::Error>;
}

impl Form for FormData {
    fn serialize(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FormData {
    case_id: CaseId,
    status: Status,
}

trait ReviewForm: Form {
    fn review_question(&mut self);
    fn submit(self) -> SubmissionResult;
}

trait ScoringForm: Form {
    fn score(self) -> impl ReviewForm;
}

trait EscalationForm: Form {}

impl EscalationForm for FormData {}

trait ClosedForm: Form {}

impl ClosedForm for FormData {}

enum SubmissionResult {
    Escalation(Box<dyn EscalationForm>),
    Closed(Box<dyn ClosedForm>),
}

impl ReviewForm for FormData {
    fn review_question(&mut self) {}

    fn submit(self) -> SubmissionResult {
        if self.case_id == CaseId(0) {
            SubmissionResult::Closed(Box::new(FormData {
                case_id: self.case_id,
                status: Status::Completed,
            }))
        } else {
            SubmissionResult::Escalation(Box::new(FormData {
                case_id: self.case_id,
                status: Status::Review,
            }))
        }
    }
}

impl ScoringForm for FormData {
    fn score(self) -> impl ReviewForm {
        FormData {
            case_id: self.case_id,
            status: Status::Review,
        }
    }
}

trait FormRepository {
    fn find_review_case(&self, case_id: CaseId) -> Option<impl ReviewForm>;
    fn find_score_case(&self, case_id: CaseId) -> Option<impl ScoringForm>;

    fn save(&self, form: &impl Form);
}

struct ConnectionPool {}

impl FormRepository for ConnectionPool {
    fn find_review_case(&self, case_id: CaseId) -> Option<impl ReviewForm> {
        let form = FormData {
            case_id,
            status: Status::Review,
        };
        Some(form)
    }

    fn find_score_case(&self, case_id: CaseId) -> Option<impl ScoringForm> {
        let form = FormData {
            case_id,
            status: Status::Scoring,
        };
        Some(form)
    }

    fn save(&self, form: &impl Form) {
        let json = form.serialize().expect("could not serialize form");
        println!("{}", json);
    }
}

#[cfg(test)]
mod tests {
    use super::ReviewForm;
    use super::ScoringForm;
    use super::{CaseId, ConnectionPool, FormRepository, SubmissionResult};

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
                let s = f.serialize().expect("could not serialize form");
                println!("{}", s);
            }
            SubmissionResult::Escalation(escalation) => println!(
                "escalation form {}",
                escalation.serialize().expect("could not serialize form")
            ),
        }
    }
}
