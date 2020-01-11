BUILD_DIR = build
TARGETS = $(addsuffix .stl,$(addprefix $(BUILD_DIR)/,tray drainer))
COMMONS = $(addsuffix .scad,constants modules)

FA = 3

.PHONY: all
all: $(TARGETS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(TARGETS): $(BUILD_DIR)/%.stl: %.scad $(BUILD_DIR) $(COMMONS)
	openscad -o $@ $< -D \$$fa=$(FA)

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
