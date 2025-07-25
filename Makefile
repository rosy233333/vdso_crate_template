# Copied and modified from https://github.com/AsyncModules/vsched/blob/e19b572714a6931972f1428e42d43cc34bcf47f2/Makefile
ARCH ?= x86_64
MODE ?= release
V ?=
PLATFORM ?=
TARGET_DIR ?= $(PWD)/target
PACKEAGE = vdso
LIB ?= libvdsoexample
RQ_CAP ?= 256
UTEST ?= user_test
UTEST_BIN ?= $(TARGET_DIR)/$(TARGET)/$(MODE)/$(UTEST)
LOG ?= error

OBJDUMP = rust-objdump -t -T -r -R -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY = rust-objcopy -X -g

# Target
ifeq ($(ARCH), x86_64)
  TARGET := x86_64-unknown-linux-musl
else ifeq ($(ARCH), aarch64)
	TARGET := aarch64-unknown-linux-musl
else ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-linux-musl
else
  $(error "ARCH" must be one of "x86_64", "riscv64" or "aarch64")
endif

OUPUT_SO := $(TARGET_DIR)/$(TARGET)/$(MODE)/libvdso.so
build_args-release := --release

ifeq ($(V),1)
  verbose := -v
else ifeq ($(V),2)
  verbose := -vv
else
  verbose :=
endif

build_args := \
	-p $(PACKEAGE) \
  -Z unstable-options \
  -Z build-std=core,compiler_builtins,alloc \
  -Z build-std-features=compiler-builtins-mem \
  --target $(TARGET) \
  --target-dir $(TARGET_DIR) \
  $(build_args-$(MODE)) \
  $(verbose)


all:
ifeq ($(wildcard $(TARGET_DIR)),)
	mkdir $(TARGET_DIR)
endif
	ARCH=${ARCH} RUSTFLAGS='-C link-arg=-fpie -C link-arg=-soname=${LIB}' cargo build $(build_args)
	@$(OBJCOPY) $(OUPUT_SO) $(OUPUT_SO)
	cp $(OUPUT_SO) $(LIB).so

disasm: all
	@$(OBJDUMP) $(OUPUT_SO)

clean:
	rm -rf $(TARGET_DIR)

utest: all
	RUST_BACKTRACE=1 RUSTFLAGS='-C target-feature=+crt-static' cargo build --bin $(UTEST) --target $(TARGET) --target-dir $(TARGET_DIR) $(build_args-$(MODE))
	RUST_LOG=$(LOG) qemu-$(ARCH) -D qemu.log -d in_asm,int,mmu,pcall,cpu_reset,page,guest_errors $(UTEST_BIN)

.PHONY: all clean 