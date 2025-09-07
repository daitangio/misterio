Follow instruction at https://packaging.python.org/en/latest/tutorials/packaging-projects/

The magic is done with

```sh
python3 -m build
python3 -m twine upload dist/*
git tag 0.1.3-dev # Tag the new version baby
```