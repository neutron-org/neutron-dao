START_DIR=$(pwd)

# ${f    <-- from variable f
#   ##   <-- greedy front trim
#   *    <-- matches anything
#   /    <-- until the last '/'
#  }
# <https://stackoverflow.com/a/3162500>

echo "generating schema for cwd-core"
cd contracts/neutron-core
cargo run --example schema > /dev/null

cd "$START_DIR"

for f in ./contracts/voting/*
do
  echo "generating schema for ${f##*/}"
  cd "$f"
  CMD="cargo run --example schema"
  eval $CMD > /dev/null
  cd "$START_DIR"
done

for f in ./contracts/proposal/*
do
  echo "generating schema for ${f##*/}"
  cd "$f"
  CMD="cargo run --example schema"
  eval $CMD > /dev/null
  cd "$START_DIR"
done

for f in ./contracts/staking/*
do
  echo "generating schema for ${f##*/}"
  cd "$f"
  CMD="cargo run --example schema"
  eval $CMD > /dev/null
  cd "$START_DIR"
done

for f in ./contracts/pre-propose/*
do
  echo "generating schema for ${f##*/}"
  cd "$f"
  CMD="cargo run --example schema"
  eval $CMD > /dev/null
  cd "$START_DIR"
done

for f in ./contracts/external/*
do
  echo "generating schema for ${f##*/}"
  cd "$f"
  CMD="cargo run --example schema"
  eval $CMD > /dev/null
  cd "$START_DIR"
done
