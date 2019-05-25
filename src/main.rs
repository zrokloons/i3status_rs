extern crate clap;
extern crate jenkins_config;
mod jenkins;

use i3monkit::*;
use i3monkit::widgets::*;
use clap::{App, Arg, crate_version};


fn main() {
    let matches = App::new("i3status_rs")
        .version(crate_version!())
        .arg(
            Arg::with_name("jenkins_config")
                .help("Path to Jenkins widget config")
                .long("jenkins-config")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let jenkins_path = matches.value_of("jenkins_config").unwrap();

    let mut statusbar = WidgetCollection::new();

    //Jenkins widget
    statusbar.push(jenkins::JenkinsWidget::new(&jenkins_path));

    //Volume widget
    statusbar.push(VolumeWidget::new("default", "Master", 0));

    //Battery status
    statusbar.push(BatteryWidget::new(0));

    //Time
    statusbar.push(DateTimeWidget::new());

    // Then start updating the satus statusbar
    statusbar.update_loop(I3Protocol::new(Header::new(1), std::io::stdout()));
}
