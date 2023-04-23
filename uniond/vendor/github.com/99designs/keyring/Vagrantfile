Vagrant.configure("2") do |config|

  config.vm.define "linux" do |linux|
    linux.vm.box = "generic/fedora32"

    linux.vm.provider "virtualbox" do |vb|
      vb.gui = true
      vb.memory = 2048
      vb.cpus = 2

      # VBoxVGA flickers constantly, use vmsvga instead which doesn't have that problem
      vb.customize ["modifyvm", :id, "--graphicscontroller", "vmsvga"]
    end

    # mount the project into /keyring
    linux.vm.synced_folder ".", "/keyring"

    # install gnome desktop and auto login
    linux.vm.provision "shell", inline: "sudo dnf install -y --exclude='gnome-initial-setup' @gnome-desktop langpacks-en"
    linux.vm.provision "shell", inline: <<-SHELL
    sudo sed -i -e 's/\\[daemon\\]/\\[daemon\\]\\nAutomaticLoginEnable=True\\nAutomaticLogin=vagrant\\n/' \
    /etc/gdm/custom.conf
    SHELL
    linux.vm.provision "shell", inline: "sudo systemctl set-default graphical.target"
    linux.vm.provision "shell", inline: "sudo systemctl isolate graphical.target"

    # set the root password - sometimes prompts show up in gnome needing to install software
    linux.vm.provision "shell", inline: "echo 'vagrant' | sudo passwd root --stdin"

    # install gnome keyring
    linux.vm.provision "shell", inline: "sudo dnf install -y gnome-keyring seahorse"

    # install kwallet
    linux.vm.provision "shell", inline: "sudo dnf install -y kwalletmanager5"

    # install pass
    linux.vm.provision "shell", inline: "sudo dnf install -y pass"

    # install golang
    linux.vm.provision "shell", inline: "sudo dnf install -y go"
  end


  config.vm.define "windows" do |windows|
    windows.vm.box = "StefanScherer/windows_10"

    windows.vm.provider "virtualbox" do |vb|
      vb.gui = true
      vb.memory = 2048
      vb.cpus = 2
    end

    # mount the project into c:\keyring
    windows.vm.synced_folder ".", "/keyring"

    # install chocolately
    windows.vm.provision "shell", privileged: true, inline: <<-SHELL
      Set-ExecutionPolicy Bypass -Scope Process -Force; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
      choco feature disable -n=showDownloadProgress
    SHELL

    # install golang
    windows.vm.provision "shell", privileged: true, inline: "choco install -y git golang"
  end

  config.vm.post_up_message = <<-MESSAGE
    There are 2 vagrant boxes:
     - linux
       - OS: Fedora 32 with Gnome Desktop
       - The keyring directory is mounted at /keyring
       - Get a shell with 'vagrant ssh linux'
       - When running go test, you'll need to use the GUI to click "Continue" on the prompts
       - After provisioning, adjusting the virtualbox GUI window size doesn't cause the resolution to update. A 'vagrant reload linux' solves the problem
     - windows
       - OS: Windows 10
       - The keyring directory is mounted at C:\keyring
       - Get a shell by starting PowerShell in the GUI
       - You can run commands remotely using 'vagrant winrm -e windows CMD'. You'll need the -e (elevated privileges) if you want to interact with wincred

    Automated scripts for running go test on vagrant boxes (run these locally):
     - ./bin/go-test-linux   - Run tests on Linux
     - ./bin/go-test-windows - Run tests on Windows
     - ./bin/go-test         - Run all tests - locally, linux and windows
  MESSAGE
end
