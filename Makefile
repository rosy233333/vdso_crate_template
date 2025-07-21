ARCH ?= x86_64
SMP ?= 1
MODE ?= release
V ?=
PLATFORM ?=
TARGET_DIR ?= $(PWD)/target
PACKEAGE = vsched
LIB ?= libvsched
RQ_CAP ?= 256
UTEST ?= init_vsched
UTEST_BIN ?= $(TARGET_DIR)/$(TARGET)/$(MODE)/$(UTEST)

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

OUPUT_SO := $(TARGET_DIR)/$(TARGET)/$(MODE)/$(LIB).so
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
  -Z build-std=core,compiler_builtins \
  -Z build-std-features=compiler-builtins-mem \
  --target $(TARGET) \
  --target-dir $(TARGET_DIR) \
  $(build_args-$(MODE)) \
  $(verbose)


all:
ifeq ($(wildcard $(TARGET_DIR)),)
	mkdir $(TARGET_DIR)
endif
	RQ_CAP=${RQ_CAP} cargo build $(build_args)
	@$(OBJCOPY) $(OUPUT_SO) $(OUPUT_SO)
	cp $(OUPUT_SO) $(LIB).so

disasm: all
	@$(OBJDUMP) $(OUPUT_SO)

clean:
	rm -rf $(TARGET_DIR)

utest: all
	RUST_BACKTRACE=1 RUSTFLAGS='-C target-feature=+crt-static' cargo build --bin $(UTEST) --target $(TARGET) --target-dir $(TARGET_DIR) $(build_args-$(MODE))
	qemu-$(ARCH) -D qemu.log -d in_asm,int,mmu,pcall,cpu_reset,page,guest_errors $(UTEST_BIN)
  

.PHONY: all clean 