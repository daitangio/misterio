#!/bin/bash
virtualenv venv
sudo apt install -y  moby-engine moby-cli
mkdir hosts/$(hostname)