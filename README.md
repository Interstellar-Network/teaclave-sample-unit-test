# incubator-teaclave-sgx-sdk sample code

Based on https://github.com/apache/incubator-teaclave-sgx-sdk/tree/master/samplecode/project_template

Created using `git filter-repo --path samplecode/project_template --path LICENSE --path=.gitignore --path edl --path common --force`

## Build

- `export CUSTOM_EDL_PATH=$PWD/edl` && `export CUSTOM_COMMON_PATH=$PWD/common`
- `(cd samplecode/project_template && make && cd bin/ && ./app)`
