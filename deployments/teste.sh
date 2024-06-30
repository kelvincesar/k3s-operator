❯ kubectl get pods -o wide                                       
NAME        READY   STATUS    RESTARTS   AGE   IP           NODE    
pod-teste   1/1     Running   0          80s   10.42.1.21   rberrypi

❯ cargo run -- --current-node rberrypi --target-node dell7580
   Running `target/debug/k3s-interface --current-node rberrypi --target-node dell7580`
[Nodes { name: "rberrypi", id: "e7193109-ab27-4aa3-bb28-2d55aabb7c7d" }, Nodes { name: "dell7580", id: "3c70e6a8-841b-4030-8bb5-c4fe09caa21d" }]
rberrypi pods: [Pods { name: "pod-teste", id: "318e04c3-b6eb-41ee-b24f-4e380d8357c2" }]
Pod 'pod-teste' deletado com sucesso
Pod 'movido-rberrypi-pod-teste' criado com sucesso no nó dell7580

❯ kubectl get pods -o wide                                   
NAME                        READY   STATUS    RESTARTS   AGE   IP           NODE    
movido-rberrypi-pod-teste   1/1     Running   0          9s    10.42.0.77   dell7580
