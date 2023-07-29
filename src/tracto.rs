use crate::{models::*, Config, Request};

#[derive(Debug)]
pub struct RequestError(String);
type RequestResult<T> = Result<T, RequestError>;

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(e: reqwest::Error) -> RequestError {
        Self(e.to_string())
    }
}

async fn make_request<T>(url: String) -> RequestResult<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    let client = reqwest::Client::new();

    let response = client.get(&url).send().await;
    if let Err(e) = &response {
        log::error!("Cannot make request to {url}: {e}")
    }

    let body = response?.json::<T>().await;
    if body.is_err() {
        log::error!(
            "Cannot deserialize response from {url} into {}",
            std::any::type_name::<T>()
        )
    }

    body.map_err(|e| e.into())
}

pub async fn fetch_schedule(cfg: &Config, request: &Request) -> RequestResult<Schedule> {
    let url = format!(
        "{}/schedule/{}/{}/{}",
        cfg.tracto_prefix, request.form, request.department, request.group
    );

    make_request::<Schedule>(url).await
}

pub async fn fetch_departments(cfg: &Config) -> RequestResult<DepartmentsList> {
    let url = format!("{}/departments", cfg.tracto_prefix);

    make_request::<DepartmentsList>(url).await
}

pub async fn fetch_exam(cfg: &Config, request: &Request) -> RequestResult<ExamList> {
    let url = format!(
        "{}/exam/{}/{}/{}",
        cfg.tracto_prefix, request.form, request.department, request.group
    );

    make_request::<ExamList>(url).await
}

pub fn find_subgroups(schedule: &Schedule) -> Vec<String> {
    let mut subgroups = schedule
        .lessons
        .iter()
        .map(|l| l.sub_group.trim().to_string())
        .filter(|sg| !sg.is_empty())
        .collect::<Vec<_>>();
    subgroups.sort_unstable();
    subgroups.dedup();
    subgroups
}

pub async fn validate_request(cfg: &Config, req: &Request) -> RequestResult<()> {
    let available_departments = fetch_departments(cfg)
        .await?
        .departments_list
        .into_iter()
        .map(|x| x.url)
        .collect::<Vec<_>>();

    if !available_departments.contains(&req.department) {
        log::error!("Incorrect department: {}.", &req.department);
        return Err(RequestError("Incorrect department".into()));
    }

    if !vec!["full", "extramural"].contains(&req.form.as_str()) {
        log::error!("Incorrect education form: {}.", &req.form.as_str());
        return Err(RequestError(
            "Incorrect education form. Should be \"full\" or \"extramural\"".into(),
        ));
    }

    let schedule = fetch_schedule(cfg, req).await?;
    let subgroups = find_subgroups(&schedule);
    if req.subgroups.iter().any(|x| !subgroups.contains(x)) {
        log::error!("Incorrect subgroup(s).");
        return Err(RequestError("Incorrect subgroup(s)".into()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn try_fetch_departments() -> RequestResult<()> {
        let cfg = Config::default();
        fetch_departments(&cfg).await?;
        Ok(())
    }

    #[actix_web::test]
    async fn try_fetch_schedule_1() -> RequestResult<()> {
        let cfg = Config::default();
        let request = Request {
            department: String::from("knt"),
            form: String::from("full"),
            group: String::from("351"),
            subgroups: vec![String::from("1_под."), String::from("цифровая_кафедра")],
            translator: false,
        };
        fetch_schedule(&cfg, &request).await?;
        Ok(())
    }

    #[actix_web::test]
    async fn try_fetch_schedule_2() -> RequestResult<()> {
        let cfg = Config::default();
        let request = Request {
            department: String::from("knt"),
            form: String::from("full"),
            group: String::from("351"),
            subgroups: Vec::new(),
            translator: false,
        };
        fetch_schedule(&cfg, &request).await?;
        Ok(())
    }

    #[actix_web::test]
    async fn try_fetch_schedule_3() -> RequestResult<()> {
        let cfg = Config::default();
        let request = Request {
            department: String::from("knt"),
            form: String::from("full"),
            group: String::from("351"),
            subgroups: vec![
                String::from("2_под."),
                String::from("цифровая_кафедра"),
                String::from("анг.ст.3"),
            ],
            translator: true,
        };
        fetch_schedule(&cfg, &request).await?;
        Ok(())
    }
}
