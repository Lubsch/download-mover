set -e

echo Save "$1":
read -rep "" -i "$HOME/" target

if [ -e "$target" ]  && [ ! -d "$target" ]; then
    echo "File already exists"
    echo
    echo Save "$1":
    read -rep "" -i "$HOME/" target
fi

if [ ! -d "$(dirname "$target")" ]; then
    echo "Directory doesnt't exist"
    echo
    echo Save "$1":
    read -rep "" -i "$HOME/" target
fi

echo -n "$target" > "$2"/"$1".download-mover
