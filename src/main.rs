use kube::{Client, api::{Api, DeleteParams, PostParams, ListParams}};
use k8s_openapi::api::core::v1::{Pod, PodSpec, Container, Affinity, NodeAffinity, NodeSelector, NodeSelectorTerm, NodeSelectorRequirement};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
    let client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");
    let pods = get_pods(&client).await;
    if pods.len() > 0 {
        // pod que queremos mover
        let pod_name = &pods[0];
        // namespace onde o pod está localizado
        let namespace = "default";
        // nome do nó destino
        let target_node = "dell7580";

        // deletar o pod existente
        delete_pod(&client, namespace, &pod_name).await?;

        // Criar um novo pod com node affinity
        create_pod_with_node_affinity(&client, namespace, &format!("novo-{}", &pod_name), target_node).await?;
    }
    

    Ok(())
}



async fn get_pods(client: &Client) -> Vec<String> {
    let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(client.clone(), "default");
    let mut pods_list: Vec<String> = Vec::with_capacity(10);
    match pods.list(&ListParams::default()).await {
        Ok(pods) => {
            for p in pods {
                let name = p.metadata.name.unwrap();
                let id = p.metadata.uid.unwrap();
                println!("Pod: {} - {}", id, name);
                pods_list.push(name.clone());

                list_pod_containers(&client, "default", &name).await;
                
            }
        },
        _ => (),
    };

    pods_list
   
}

async fn delete_pod(client: &Client, namespace: &str, pod_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);

    match pods.delete(pod_name, &DeleteParams::default()).await {
        Ok(_) => {
            println!("Pod {} deletado com sucesso", pod_name);
        },
        Err(e) => {
            eprintln!("Erro ao deletar o pod {}: {}", pod_name, e);
        },
    };

    Ok(())
}

async fn create_pod_with_node_affinity(client: &Client, namespace: &str, pod_name: &str, node_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let pod = Pod {
        metadata: ObjectMeta {
            name: Some(pod_name.to_string()),
            ..Default::default()
        },
        spec: Some(PodSpec {
            containers: vec![Container {
                name: "my-container".to_string(),
                image: Some("nginx:1.14.2".to_string()),
                ..Default::default()
            }],
            affinity: Some(Affinity {
                node_affinity: Some(NodeAffinity {
                    required_during_scheduling_ignored_during_execution: Some(NodeSelector {
                        node_selector_terms: vec![NodeSelectorTerm {
                            match_expressions: Some(vec![NodeSelectorRequirement {
                                key: "kubernetes.io/hostname".to_string(),
                                operator: "In".to_string(),
                                values: Some(vec![node_name.to_string()]),
                            }]),
                            ..Default::default()
                        }],
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let pp = PostParams::default();
    match pods.create(&pp, &pod).await {
        Ok(_) => {
            println!("Pod {} criado com sucesso no nó {}", pod_name, node_name);
        },
        Err(e) => {
            eprintln!("Erro ao criar o pod {}: {}", pod_name, e);
        },
    };

    Ok(())
}

async fn list_pod_containers(client: &Client, namespace: &str, pod_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);

    match pods.get(pod_name).await {
        Ok(pod) => {
            match pod.spec {
                Some(spec) => {
                    for container in spec.containers {
                        println!("\t* Container Name: {}", container.name);
                        if let Some(image) = container.image {
                            println!("\t* Image: {}", image);
                        }
                        if let Some(command) = container.command {
                            println!("\t* Command: {:?}", command);
                        }
                        if let Some(args) = container.args {
                            println!("\t* Args: {:?}", args);
                        }
                    }
                }
                None => eprintln!("No spec!")
            }
        },
        Err(e) => {
            eprintln!("Erro ao obter o pod {}: {}", pod_name, e);
        },
    };

    Ok(())
}