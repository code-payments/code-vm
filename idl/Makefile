# A makefile for building the program and updating the discriminators in the IDL
# to match the real program values. Anchor uses SHA256 hashes to identify the
# discriminators, we use steel, which is a more human-readable format.

ANCHOR_BUILD=anchor build --skip-lint

UPDATE_IDL=bun ./src/scripts/update-discriminators.ts > ./src/target/idl/code_vm_updated.json
COPY_IDL=cp -f ./src/target/idl/code_vm_updated.json ./src/target/idl/code_vm.json
MOVE_IDL=cp -f ./src/target/idl/code_vm_updated.json ./code_vm.json

anchor_build:
	cd src && $(ANCHOR_BUILD)

update_idl:
	@ cd src && npm install
	@ $(UPDATE_IDL)

copy_idl:
	@ $(COPY_IDL)

move_idl:
	@ $(MOVE_IDL)

idl: anchor_build update_idl copy_idl move_idl