use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Container, Namespace, Pod};
use k8s_openapi::chrono;
use kube::{Api, Error, ResourceExt};
use kube::api::{ListParams, Patch, PatchParams};
use serde_json::json;
use futures::{AsyncBufReadExt, TryStreamExt};

pub struct App {
    client: kube::Client,
}

impl App {
    pub async fn new() -> Result<App, Error> {
        let client = kube::Client::try_default().await?;
        Ok(App { client })
    }
    pub(crate) async fn get_namespaces(&self) -> Result<Vec<Namespace>, Error> {
        let namespaces: Api<Namespace> = Api::all(self.client.clone());
        let lp = ListParams::default();
        let mut result = Vec::new();
        for ns in namespaces.list(&lp).await? {
            result.push(ns);
        }
        Ok(result)
    }
    pub(crate) async fn get_deployments(&self, namespace:&str) -> Result<Vec<Deployment>, Error> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &namespace);
        let lp = ListParams::default();
        let mut result = Vec::new();
        for dep in deployments.list(&lp).await? {
            result.push(dep);
        }
        Ok(result)
    }
    pub(crate) async fn get_pods(&self, namespace: &str) -> Result<Vec<Pod>, Error> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let lp = ListParams::default();
        let mut result = Vec::new();
        for pod in pods.list(&lp).await? {
            result.push(pod);
        }
        Ok(result)
    }
    pub(crate) async fn get_pod_containers(&self, pod: &Pod) -> Result<Vec<String>, Error> {
        let containers = pod.spec.clone().unwrap().containers;
        let mut container_names = Vec::new();
        for container in containers {
            container_names.push(container.name.clone());
        }
        Ok(container_names)
    }
    pub(crate) async fn get_pod_container_logs(&self, pod: &Pod, container: &str) -> Result<(), Error> {
        let namespace = pod.namespace().unwrap_or("default".to_string());
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &namespace);
        let lp = ListParams::default();
        for p in pods.list(&lp).await? {
            if p.name_any() == pod.name_any() {
                let mut logs = pods.log_stream(&p.name_any(),  &Default::default()).await?.lines();
                // print the logs
                while let Some(line) = logs.try_next().await.unwrap() {
                    println!("{}", line);
                }
            }
        }
        Ok(())
    }
    pub(crate) async fn restart_deployment(&self, deployment: Deployment) -> Result<(), Error> {
        let namespace = deployment.namespace().unwrap_or("default".to_string());
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &namespace);
        let patch_spec = self.get_patch_spec().await;
        let pp = PatchParams::apply(&namespace);
        deployments.patch(&deployment.name_any(), &pp,&Patch::Merge(&patch_spec)).await?;
        Ok(())
    }
    pub(crate) async fn delete_deployment (&self, deployment: Deployment) -> Result<(), Error> {
        let namespace = deployment.namespace().unwrap_or("default".to_string());
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &namespace);
        deployments.delete(&deployment.name_any(), &Default::default()).await?;
        Ok(())
    }
    pub(crate) async fn delete_pod(&self, pod: Pod) -> Result<(), Error> {
        let pods: Api<Pod> = Api::all(self.client.clone());
        pods.delete(&pod.name_any(), &Default::default()).await?;
        Ok(())
    }
    async fn get_patch_spec(&self) -> serde_json::Value {
        let now = chrono::Utc::now().to_rfc3339();
        json!({
            "spec": {
                "template": {
                    "metadata": {
                        "annotations": {
                            "kubectl.kubernetes.io/restartedAt": now,
                        },
                    },
                },
            },
        })
    }
}