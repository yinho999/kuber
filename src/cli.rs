use clap::{Parser, Subcommand};
use dialoguer::console::Term;
use k8s_openapi::chrono;
use kube::ResourceExt;
use crate::App;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Get a list of namespaces
    GetNamespaces,
    /// Get a list of pods
    GetPods,
    /// Get a list of deployments
    GetDeployments,
    /// Restart a deployment
    RestartDeployment,
    /// Logs for a pod container
    Logs,
    /// Delete a pod
    DeletePod,
    /// Delete a deployment
    DeleteDeployment,
}

impl Cli {
    pub async fn run(&self, app: App) -> Result<(), kube::Error> {
        match self.subcmd {
            SubCommands::GetNamespaces=>{
                let namespaces = app.get_namespaces().await?;
                for namespace in namespaces {
                    println!("{}", namespace.name_any());
                }
            }
            SubCommands::GetPods => {
                let pods = app.get_pods("default").await?;
                for pod in pods {
                    println!("{}", pod.name_any());
                }
            }
            SubCommands::GetDeployments => {
                let deployments = app.get_deployments("default").await?;
                for deployment in deployments {
                    println!("{}", deployment.name_any());
                }
            }
            SubCommands::Logs => {
                let pods = app.get_pods("default").await?;
                let mut pod_names = Vec::new();
                for pod in &pods {
                    pod_names.push(pod.name_any());
                }
                let pod_name = dialoguer::Select::new()
                    .with_prompt("Select a pod")
                    .items(&pod_names)
                    .default(0)
                    .interact_on_opt(&Term::stderr()).unwrap();

                let pod = &pods[pod_name.unwrap()];
                let containers = app.get_pod_containers(pod).await?;
                let mut container_names = Vec::new();
                for container in &containers {
                    container_names.push(container.clone());
                }
                let container_name = dialoguer::Select::new()
                    .with_prompt("Select a container")
                    .items(&container_names).default(0)
                    .interact_on_opt(&Term::stderr()).unwrap();
                let container = &containers[container_name.unwrap()];
                app.get_pod_container_logs(pod, container).await?;
            }
            SubCommands::RestartDeployment => {
                let deployments = app.get_deployments("default").await?;
                let mut deployment_names = Vec::new();
                for deployment in &deployments {
                    let name = deployment.name_any();
                    let ready = deployment.status.as_ref().unwrap().ready_replicas.as_ref().unwrap_or(&0);
                    let up_to_date = deployment.status.as_ref().unwrap().updated_replicas.as_ref().unwrap_or(&0);
                    let available = deployment.status.as_ref().unwrap().available_replicas.as_ref().unwrap_or(&0);
                    let age = deployment.metadata.creation_timestamp.as_ref().unwrap();
                    let age = chrono::Utc::now().signed_duration_since(age.0);
                    let age = format!("{}d {}h {}m {}s", age.num_days(), age.num_hours() % 24, age.num_minutes() % 60, age.num_seconds() % 60);
                    deployment_names.push(format!("{: <30} {: <10} {: <10} {: <10} {: <10}", name, ready, up_to_date, available, age));
                }
                let selection = dialoguer::Select::new()
                    .with_prompt("Select a deployment to restart")
                    .items(&deployment_names).default(0)
                    .interact_on_opt(&Term::stderr()).unwrap();
                let deployment = deployments[selection.unwrap()].clone();
                app.restart_deployment(deployment).await?;
            }
            SubCommands::DeletePod => {
                println!("Deleting a pod");
            }
            SubCommands::DeleteDeployment => {
                println!("Deleting a deployment");
            }
        }
        Ok(())
    }
}


