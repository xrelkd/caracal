{
  runCommand,
  installShellFiles,
  caracal,
}:

runCommand "caracal-completions"
  {
    nativeBuildInputs = [ installShellFiles ];
  }

  ''
    for cmd in caracal caracal-daemon caracal-tui; do
      installShellCompletion --cmd $cmd \
        --bash <(${caracal}/bin/$cmd completions bash) \
        --fish <(${caracal}/bin/$cmd completions fish) \
        --zsh  <(${caracal}/bin/$cmd completions zsh)
    done
  ''
