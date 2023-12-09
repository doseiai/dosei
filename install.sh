dosei_install="${DOSEI_INSTALL:-$HOME/.dosei}"
bin_dir="$dosei_install/bin"
exe="$bin_dir/dosei"

if [ ! -d "$bin_dir" ]; then
  mkdir -p "$bin_dir"
fi

cp ~/Documents/alw3ys/dosei/target/release/dosei "$bin_dir"
chmod +x "$exe"

echo "Dosei was installed successfully to $exe"

if command -v dosei >/dev/null; then
  echo "Run 'dosei --help' to get started"
else
	case $SHELL in
	/bin/zsh) shell_profile=".zshrc" ;;
	*) shell_profile=".bashrc" ;;
	esac
	echo "Manually add the directory to your \$HOME/$shell_profile (or similar)"
	echo "  export DOSEI_INSTALL=\"$dosei_install\""
	echo "  export PATH=\"\$DOSEI_INSTALL/bin:\$PATH\""
	echo "Run '$exe --help' to get started"
fi
echo
echo "Stuck? Join our Discord https://discord.com/invite/BP5aUkhcAh"

