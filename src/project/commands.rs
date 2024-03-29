use crate::project::project_args::{
    ProjectCommand,
    ProjectSubCommand,
    CreateProject,
};
use std::{
    env,
    io::{BufRead, BufReader, Write, BufWriter},
    process::{Command, exit},
    fs::{self, File},
    // process::exit,
    path::{Path, PathBuf},
};

use clap::error::ContextValue::String;
use dialoguer::Input;


pub fn handle_project_command(project: ProjectCommand) -> () {
    let command = project.command;
    match command {
        ProjectSubCommand::Create(project) => {
            create_project(project);
        }
    }
}

pub fn create_project(project: CreateProject) -> () {
    println!("Creating a Django Project {:?}", project);
    // check os type
    if cfg!(target_os = "windows") {
        create_windows_project(&project);
    }
    create_linux_project(&project);
    // let name = match(Input::new().with_prompt("What is your name?").interact()){
    //     Ok(result) => result,
    //     _ => "Give a valid name".to_string()
    // };
    // println!("Hello, {name}");
}

pub fn create_windows_project(_project: &CreateProject) -> () {
    return;
}

pub fn create_linux_project(project: &CreateProject) -> () {
//     TODO deactivate any running virtual environments
//     check if python installed
    let is_windows = false;
    let python_installed = check_python_installed(is_windows);
    if python_installed == false {
        eprintln!("Install Python in your device to continue");
        return;
    }
    // python is installed
    // pip install virtualenv
    let virtualenv_installed = check_virtualenv_installed(is_windows);
    if virtualenv_installed == false {
        eprintln!("Installing virtualenv python package");
        let result = install_virtualenv(is_windows);
        if result == false {
            return;
        }
    }
    // create the project directory
    create_project_directory(&project.name);

    // copy dockerfile to the project directory
    add_dockerfile(&project.name);
    // copy requirements.txt to project directory
    add_requirements_txt(&project.name);

    // get in the folder
    if let Err(_error) = std::env::set_current_dir(&project.name) {
        eprintln!("Couldn't change directory to the project");
        return;
    }

    // create the venv folder
    let result = create_virtual_env(is_windows);
    if result == false {
        return;
    }

    // create the django project with django-admin
    let _result = create_django_project(is_windows, &project.name);
    if result == false {
        return;
    }

    // add settings file
    add_settings_py_file(&project.name);

    // add project urls.py
    add_project_urls_file(&project.name);

    // add virtual environment
    add_dot_env_file();

    // add users app
    add_users_app();

}

fn remove_file<P: AsRef<Path>>(file_name: &P) -> bool {
//     check if file exists
    if !fs::metadata(file_name).is_ok() {
        // exit(1);
        return true;
    }
    if let Err(error) = fs::remove_file(file_name) {
        eprintln!("Failed to remove the file.\nError: {}", error);
        // exit(1);
        return false;
    }
    return true;
}


fn add_users_app() -> () {
    let project_dir = env::current_dir();
    if let Err(_error) = project_dir {
        eprintln!("Couldn't get the cwd");
        exit(1);
    }

    let django_users_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("users");
    let destination_dir = &project_dir.unwrap().join("users");

    if let Err(_error) = fs::create_dir_all(&destination_dir) {
        eprintln!("An error occured {:?}", _error);
        exit(1);
    }

    if !django_users_dir.exists() {
        eprintln!("Ensure that you have the django users app");
        exit(1);
    }

    let users_files = match fs::read_dir(django_users_dir) {
        Ok(values) => values,
        _ => {
            eprintln!("Couldn't read user app files");
            exit(1);
        }
    };

    for entry in users_files {
        if let Ok(entry) = entry {
            let entry = entry;
            let entry_path = entry.path();

            if let Err(_error) = fs::copy(&entry_path, destination_dir.
                join(&entry_path.
                    file_name().
                    unwrap().
                    to_string_lossy().
                    to_string())) {
                eprintln!("Something went wrong while copying {:?}", entry_path.file_name());
            }
        }
    }

    let project_dir = env::current_dir();

    // copy serializers.py
    let serializers_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("serializers.py");
    let destination_dir = project_dir.unwrap().join("users").join("serializers.py");
    if let Err(_error) = fs::copy(serializers_file, destination_dir) {
        eprintln!("Something went wrong file copying serializers.py file");
    }

    let project_dir = env::current_dir();

    // copy urls.py
    let urls_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("urls.py");
    let destination_dir = project_dir.unwrap().join("users").join("urls.py");
    if let Err(_error) = fs::copy(urls_file, destination_dir) {
        eprintln!("Something went wrong file copying serializers.py file");
    }

    let project_dir = env::current_dir();

    // copy filters.py
    let filters_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("filters.py");
    let destination_dir = project_dir.unwrap().join("users").join("filters.py");
    if let Err(_error) = fs::copy(filters_file, destination_dir) {
        eprintln!("Something went wrong file copying serializers.py file");
    }

    println!("Successfully created a django app, users");
}

fn add_dot_env_file() -> () {
    let project_dir = env::current_dir();
    if let Err(_error) = project_dir {
        eprintln!("Couldn't get the cwd");
        exit(1);
    }

    let my_env_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("my_env");
    let destination_dir = project_dir.unwrap().join(".env");
    if let Err(_error) = fs::copy(my_env_file, destination_dir) {
        eprintln!("Couldn't copy the .env file to the project");
        exit(1);
    }
    println!("Successfully copied the .env file");
}

fn add_project_urls_file(project_name: &str) -> (){
    if let Ok(project_dir) = env::current_dir(){
        let urls_file = &project_dir.join(project_name).join("urls.py");
        if remove_file(urls_file) == false {
            eprintln!("Error occurred deleting the projects urls.py file");
            return;
        }

        let django_proj_urls_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("proj_urls.py");
        let target_urls_file = &project_dir.join(project_name).join("urls.py");

        if let Err(error) = fs::copy(django_proj_urls_file, target_urls_file){
            eprintln!("An error occurred copying the django urls.py file, {:?}", error);
            return;
        }
        println!("Successfully added the urls.py file");
        return;
    }
}

fn edit_settings_py_file(project_name: &str, django_settings: &PathBuf) -> () {
    if let Ok(django_settings_file) = File::open(django_settings) {
        let django_settings_reader = BufReader::new(django_settings_file);

        if let Ok(project_dir) = env::current_dir() {
            let new_settings_path = project_dir.join(project_name).join("settings.py");
            if let Ok(settings_file) = File::create(new_settings_path) {
                let mut settings_writer = BufWriter::new(settings_file);

                for value in django_settings_reader.lines() {
                    if let Ok(line) = value {
                        // add the users app
                        if line.contains("##django apps##") {
                            let new_line = line.replace("##django apps##", "'users',\n\t##django apps##");
                            if let Err(error) = writeln!(settings_writer, "{}", new_line) {
                                eprintln!("Error occurred while adding users app {:?}", error);
                            }
                            continue;
                        }

                        // configure project name
                        if line.contains("project_name") {
                            let new_line = line.replace("project_name", format!("{}", project_name).as_str());
                            if let Err(error) = writeln!(settings_writer, "{}", new_line) {
                                eprintln!("Error occurred while configuring project name {:?}", error);
                            }
                            continue;
                        }

                        if let Err(error) = writeln!(settings_writer, "{}", line) {
                            eprintln!("Error occurred while changing root urlconf {:?}", error);
                        }
                    }
                }

                //  ensure all changes are written to the file
                if let Err(error) = settings_writer.flush() {
                    eprintln!("Error occurred while editing settings file {:?}", error);
                }
            }
        }
    }
}

fn add_settings_py_file(project_name: &str) -> () {
    // remove default settings py file
    let project_dir = env::current_dir();
    if let Err(_error) = project_dir {
        eprintln!("Couldn't get the cwd");
        return;
    }
    let settings_file = project_dir.unwrap().join(project_name).join("settings.py");
    if remove_file(&settings_file) == false {
        return;
    }

    let django_settings_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("settings.py");

    edit_settings_py_file(project_name, &django_settings_file);
}

fn create_django_project(is_windows: bool, project_name: &str) -> bool {
    if is_windows == false {
        let child = Command::new("django-admin")
            .args(["startproject", project_name, "."])
            .spawn()
            .ok();
        if let Some(mut result) = child {
            let output = result.wait().ok();
            if let Some(result) = output {
                if result.success() == true {
                    println!("{} has been created successfully", project_name);
                } else {
                    eprintln!("Something went wrong while creating the django project {}", project_name);
                }
                return result.success();
            }
            eprintln!("Something went wrong while creating the django project {}", project_name);
            return false;
        }
        eprintln!("Something went wrong while creating the django project {}", project_name);
        return false;
    }
    eprintln!("This command works on linux computers only");
    return false;
}

fn create_virtual_env(is_windows: bool) -> bool {
    if !fs::metadata("venv").is_ok() {
        if is_windows == false {
            let child = Command::new("python3")
                .args(["-m", "virtualenv", "venv"])
                .spawn();
            if let Ok(mut result) = child {
                let output = result.wait();
                if let Ok(result) = output {
                    if result.success() == true {
                        println!("Successfully created virtual envs");
                    } else {
                        eprintln!("Something went wrong while creating virtual envs");
                    }
                    return result.success();
                }
                eprintln!("Something went wrong while creating virtual envs\
                \nError: {:?}
                ", output.err());
                return false;
            }
            eprintln!("Something went wrong while creating virtual envs\
            \nError: {:?}", child.err());
            return false;
        }
        return false;
    }
    println!("Virtual environment folder exists");
    return false;
}

fn add_requirements_txt(project_name: &str) -> () {
    let docker_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("requirements.txt");
    let result = env::current_dir();

    if let Err(_error) = result {
        return;
    }
    let destination_dir = result.unwrap().join(project_name).join("requirements.txt");

    // copy file
    if let Err(_error) = fs::copy(docker_file, destination_dir) {
        eprintln!("Failed to copy the requirements.txt to the project");
        return;
    }
    println!("Successfully copied the requirements.txt");
    return;
}

fn add_dockerfile(project_name: &str) -> () {
    let docker_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("Dockerfile");
    let docker_compose_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("django").join("docker-compose.yml");
    let result = env::current_dir();

    if let Err(_error) = result {
        // exit(1);
        return;
    }

    // copy dockerfile
    let destination_dir = result.unwrap().join(project_name).join("Dockerfile");
    if let Err(_error) = fs::copy(docker_file, destination_dir) {
        eprintln!("Failed to copy the Dockerfile to the project");
        // exit(1);
        return;
    }

    // copy docker compose
    let result = env::current_dir();
    let destination_dir = result.unwrap().join(project_name).join("docker-compose.yml");
    if let Err(_error) = fs::copy(docker_compose_file, destination_dir) {
        eprintln!("Failed to copy the docker-compose.yml file to the project");
        // exit(1);
        return;
    }
    println!("Successfully copied the Dockerfile and docker-compose.yml");
    return;
}

fn create_project_directory(project_name: &str) -> () {
    if !fs::metadata(project_name).is_ok() {
        if let Err(err) = fs::create_dir(project_name) {
            println!("Failed to created django project {}", err);
            exit(1);
        }
        println!("Project folder {} created successfully", project_name);
    }
    println!("Project folder {} already exists", project_name);
}


fn install_virtualenv(is_windows: bool) -> bool {
    if is_windows == false {
        let child = Command::new("python3")
            .args(["-m", "pip", "install", "virtualenv"])
            .spawn()
            .ok();
        if let Some(mut result) = child {
            let status = result.wait().ok();
            if let Some(result) = status {
                if result.success() == true {
                    println!("Virtualenv installed successfully");
                    return true;
                }
            }
            eprintln!("Something went wrong while installing virtualenv");
            return false;
        }

        eprintln!("Something went wrong while installing virtualenv");
        return false;
    }
    eprintln!("This command works on linux computers only");
    return false;
}

fn check_virtualenv_installed(is_windows: bool) -> bool {
    if is_windows == false {
        let child = Command::new("sh")
            .args(["-c", "command", "-v", "virtualenv"])
            .spawn()
            .ok();

        if let Some(mut result) = child {
            let status = result.wait().ok();
            if let Some(result) = status {
                return result.success();
            }
        }
        return false;
    }
    eprintln!("This command works on linux computers only");
    return false;
}

fn check_python_installed(is_windows: bool) -> bool {
    if is_windows == false {
        let child = Command::new("which")
            .arg("python3")
            .spawn()
            .ok();

        if let Some(mut result) = child {
            let status = result.wait().ok();
            if let Some(result) = status {
                return result.success();
            }
            return false;
        }
        eprintln!("Something went wrong");
        return false;
    }
    eprintln!("This command works on linux computers only");
    return false;
}

