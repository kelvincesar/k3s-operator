use kube::{Client, api::{Api, DeleteParams, PostParams, ListParams}};
use k8s_openapi::api::core::v1::{Pod, PodSpec, Container, Affinity, NodeAffinity, NodeSelector, NodeSelectorTerm, NodeSelectorRequirement};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use tokio;
use clap::Parser;



#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Nome do nó atual de onde o pod está sendo executado
    #[arg(short, long)]
    current_node: String,

    /// Nome do nó destino para onde o pod será realocado
    #[arg(short, long)]
    target_node: String,
}

// export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let Args { current_node, target_node } = args;

    let client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    // namespace onde o pod está localizado
    let namespace = "default";
    
    let nodes = get_nodes(&client).await;
    let pods = get_pods(&client, &current_node).await;

    println!("{:?}", nodes);
    println!("{} pods: {:?}", current_node, pods);

    if pods.len() > 0 {
        // pod que queremos mover
        let pod_name = &pods[0].name;

        // deletar o pod existente
        delete_pod(&client, namespace, &pod_name).await?;

        // Criar um novo pod com node affinity
        let new_name = new_pod_name(&pod_name, &current_node);
        create_pod_with_node_affinity(&client, namespace, &new_name, &target_node).await?;
    } else {
        println!("Nenhum pod encontrado para mover.");
        let pod_name = "pod-teste";
        create_pod_with_node_affinity(&client, namespace, pod_name, &target_node).await?;
    }
    
    Ok(())
}

fn new_pod_name(current_name: &str, node: &str) -> String {
    let name = current_name.strip_prefix("movido-").unwrap_or(current_name);
    format!("movido-{}-{}", node, name)
}

#[derive(Debug)]
#[allow(dead_code)]
struct Pods {
   name: String,
   id: String,
}
async fn get_pods(client: &Client, node_name: &str) -> Vec<Pods> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");
    let mut pods_list: Vec<Pods> = Vec::with_capacity(10);

    // ListParams com selector para filtrar por node
    let lp = ListParams::default()
        .fields(&format!("spec.nodeName={}", node_name));

    match pods.list(&lp).await {
        Ok(pod_list) => {
            for p in pod_list.items {
                let name = p.metadata.name.clone().unwrap();
                let id = p.metadata.uid.clone().unwrap();
                // println!("Pod: {} - {}", id, name);
                pods_list.push(Pods { name: name.clone(), id });

                if let Err(e) = list_pod_containers(&client, "default", &name).await {
                    println!("Erro ao listar os containers do pod {}: {}", name, e);
                }
            }
        },
        Err(e) => println!("Erro ao listar os pods: {}", e),
    };

    pods_list
}

#[derive(Debug)]
#[allow(dead_code)]
struct Nodes {
    name: String,
    id: String,
}
async fn get_nodes(client: &Client) -> Vec<Nodes> {
    let nodes: Api<k8s_openapi::api::core::v1::Node> = Api::all(client.clone());
    let mut nodes_list: Vec<Nodes> = Vec::with_capacity(10);
    match nodes.list(&ListParams::default()).await {
        Ok(nodes) => {
            for n in nodes {
                let name = n.metadata.name.unwrap();
                let id = n.metadata.uid.unwrap();
                //println!("Node: {} - {}", id, name);
                nodes_list.push(Nodes { name: name.clone(), id });
            }
        },
        _ => (),
    };


    nodes_list
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
                name: "generated-container".to_string(),
                image: Some("bashofmann/rancher-demo:1.0.0".to_string()),
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
                        //println!("\t* Container Name: {}", container.name);
                        if let Some(_image) = container.image {
                            //println!("\t* Image: {}", image);
                        }
                        if let Some(_command) = container.command {
                            //println!("\t* Command: {:?}", command);
                        }
                        if let Some(_args) = container.args {
                            //println!("\t* Args: {:?}", args);
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