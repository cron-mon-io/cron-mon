#! /usr/bin/bash

# Need bash-completion for bash completion, curl to download startship.rs, and ssh for Git.
apt-get update && apt-get install -y bash-completion curl ssh

# Setup bash completion
echo "source /etc/bash_completion" >> ~/.bashrc

# Setup starship prompt
curl -sS https://starship.rs/install.sh | sh -s -- -y
echo 'eval "$(starship init bash)"' >> ~/.bashrc

# Set VS Code as default editor
echo 'export EDITOR="code "' >> ~/.bashrc
