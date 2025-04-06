# tummycrypt

TummyCrypt is experimental storage infrastructure for "friendlabbing"" based on seaweedfs.

# Objectives:


**to deploy:**
```
# deploy to triplicated masters: 
ansible-playbook -i hosts/ -K masters.yml -u "jess"

# deploy a volume: 
ansible-playbook -i hosts/ -K volumes.yml -u "jess"

# deploy a filer: 
ansible-playbook -i hosts/ -K filers.yml -u "jess"
```