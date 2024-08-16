#!/bin/bash

ip=60.205.143.45:50000

current_path=$(pwd)

folder_num=$(ls -lA | grep "^d" | wc -l)

for ((i=1;i<folder_num;i++)); do

	sed -i "3s/.*/  \"node_addr\":\"$ip\",/" "$current_path/node$i/config/config_file/node_config.json"
done
