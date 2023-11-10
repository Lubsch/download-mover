choose() {
  echo Save "$bname":
  read -rep "" -i "$dir/" input

  if [ -d "$input" ]; then
    target="$input/$bname"
    return
  fi

  if [ -e "$target" ]; then
      echo "File already exists"
      echo
      choose
      return
  fi

  if [ -d "$(dirname "$target")" ]; then
      target="$input"
      return
  fi

  echo "Directory doesnt't exist"
  echo
  choose
}

bname=$(basename "$1")
if [ -e /tmp/lastchoice.download-mover ]; then
  dir=$(cat /tmp/lastchoice.download-mover)
else
  dir="$HOME"
fi

choose

echo -n "$(dirname "$target")" > /tmp/lastchoice.download-mover
echo -n "$target" > "$1".download-mover
