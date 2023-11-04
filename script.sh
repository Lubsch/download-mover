set -e

bname=$(basename "$1")
echo Save "$bname":
read -rep "" -i "$HOME/" target

if [ -e "$target" ]  && [ ! -d "$target" ]; then
    echo "File already exists"
    echo
    echo Save "$bname":
    read -rep "" -i "$HOME/" target
fi

if [ ! -d "$(dirname "$target")" ]; then
    echo "Directory doesnt't exist"
    echo
    echo Save "$bname":
    read -rep "" -i "$HOME/" target
fi

echo -n "$target" > "$1".download-mover
