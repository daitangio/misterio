Follow instruction at https://packaging.python.org/en/latest/tutorials/packaging-projects/

The magic is done with

```sh
pip install build
python3 -m build twine
python3 -m twine upload dist/*
git tag 0.1.5 # Tag the new version baby
```