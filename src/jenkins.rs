use i3monkit::{Block, Widget, WidgetUpdate};
use jenkins_api::{Jenkins, JenkinsBuilder};
use jenkins_api::build::{BuildStatus, CommonBuild, ShortBuild};
use serde::{Deserialize};
use std::{fs, time};


// Struct holding information when building up Pango markup string
struct PangoMarkup {

    // An RGB color spec such as '#00FF00' or a color name such as 'red'
    foreground: String,

    // An RGB color spec such as '#00FF00' or a color name such as 'red'
    background: String,

    // Text to be visible
    text: String
}

impl Default for PangoMarkup {

    fn default() -> PangoMarkup {
        PangoMarkup {
            foreground: String::from("white"),
            background: String::from("black"),
            text: String::from(""),
        }
    }
}

impl PangoMarkup {

    // Constructor
    fn new() -> PangoMarkup {
        PangoMarkup { ..Default::default() }
    }

    // Setter of foreground field
    fn set_foreground(mut self, foreground: &str) -> Self {
        self.foreground = String::from(foreground);
        self
    }

    // Setter of background field
    fn set_background(mut self, background: &str) -> Self {
        self.background = String::from(background);
        self
    }

    // Setter of text field
    fn set_text(mut self, text: &str) -> Self {
        self.text = String::from(text);
        self
    }

    // build method that construct the resulting Pango markup string
    fn build(&self) -> String {
        let mut tmp = String::new();
        tmp.push_str("<span foreground='");
        tmp.push_str(self.foreground.as_str());
        tmp.push_str("' background='");
        tmp.push_str(self.background.as_str());
        tmp.push_str("'>");
        tmp.push_str(self.text.as_str());
        tmp.push_str("</span>");
        tmp
    }
}


#[derive(Deserialize)]
pub struct JenkinsJobs {

    // url of Jenkins instance including port
    pub jenkins: String,

    // name to be shown in the bar
    pub name: String,

    // list of names of jobs that should be tracked
    pub jobs: Vec<String>,
}

#[derive(Deserialize)]
pub struct JenkinsWidget {

    // List of Jenkins jobs
    jobs: Vec<JenkinsJobs>,

    // Update frequency (seconds)
    update_frequency: u64,
}

impl JenkinsWidget {

    // Create a JenkinsWidget struct
    pub fn new(path: &str) -> JenkinsWidget {

        // Read the config into a String
        let data = match fs::read_to_string(&path) {
            Ok(data) => data,
            Err(error) => {
                panic!("Error reading file: {}, {}", path, error);
            },
        };

        // deserialize contents into JenkinsWidget struct
        match serde_yaml::from_str(&data) {
            Ok(jenkins_widget) => jenkins_widget,
            Err(error) => {
                panic!("Error deserialize config: {}, {}", path, error);
            },
        }
    }

    // Get the last build of a given job on a Jenkins instance. If job is not
    // available or the job does not have a last_build, just return None back
    fn last_build(&self, jenkins: &Jenkins, job: &str) -> Option<CommonBuild> {

        // Get the job struct if available
        match jenkins.get_job(job) {
            Ok(this_job) => {

                // Test if last_build field is present
                match &this_job.last_build {
                    Some(lb) => self.get_build(&jenkins, job, &lb),
                    None => None,
                }
            },
            Err(_) => None,
        }
    }

    // Get a Jenkins build and return
    fn get_build(&self, jenkins: &Jenkins, job: &str, build: &ShortBuild) ->
        Option<CommonBuild> {

        match jenkins.get_build(job, build.number) {
            Ok(build) => Some(build.to_owned()),
            Err(_) => None,
        }
    }
}

impl Widget for JenkinsWidget {

    fn update(&mut self) -> Option<WidgetUpdate> {
        let mut data = Vec::new();

        for tracked in &self.jobs {
            let mut connected:bool = false;
            let mut extra = Vec::new();

            for job in &tracked.jobs {
                // Get last build from Jenkins
                let jenkins_builder = JenkinsBuilder::new(&tracked.jenkins);
                let jenkins = match jenkins_builder.build() {
                    Ok(jenkins) => jenkins,
                    Err(_)      => return None,
                };

                if let Some(build) = self.last_build(&jenkins, &job) {
                    // Indicator that we were able to connect to Jenkins
                    connected = true;

                    // Depending on the current status of last build colorize
                    // it and add it to the Extra section
                    if build.building {
                        extra.push(PangoMarkup::new()
                                   .set_background("blue")
                                   .set_text(&build.full_display_name)
                                   .build());
                    }
                    else {
                        match build.result {
                            Some(BuildStatus::Failure) => {
                                extra.push(PangoMarkup::new()
                                           .set_background("red")
                                           .set_text(&build.full_display_name)
                                           .build());
                            },
                            Some(_) | None => (),
                        }
                    }
                }
            }

            // Add the name with color depending on if we were able to get a job
            // from the tracked jobs for this name or not.
            #[allow(clippy::match_bool)]
            match connected {
                true  => data.push(
                    PangoMarkup::new()
                        .set_text(&tracked.name)
                        .build()),
                false => data.push(
                    PangoMarkup::new()
                        .set_foreground("grey")
                        .set_text(&tracked.name)
                        .build()),
            };

            // Only push the extra separator if we have extra information
            if !extra.is_empty() {
                data.push(extra.join(
                        &PangoMarkup::new()
                            .set_text("|")
                            .build()));
            }
        }

        // Create the i3monkit block and add collected data
        let mut block = Block::new();
        block.append_full_text(&data.join(
                &PangoMarkup::new()
                    .set_text(" ")
                    .build()));

        block.use_pango();

        Some(WidgetUpdate{
            refresh_interval: time::Duration::new(self.update_frequency, 0),
            data: Some(block)
        })
    }
}
