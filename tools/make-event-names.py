#!/usr/bin/env python
# Parses linux/input.h scanning for #define KEY_FOO 134
# Prints Rust source header files that can be used for
# mapping and lookup tables.
#
# The original version of this file is in libevdev
#

from __future__ import print_function
import re
import sys

class Bits(object):
	pass

prefixes = [
		"EV_",
		"REL_",
		"ABS_",
		"KEY_",
		"BTN_",
		"LED_",
		"SND_",
		"MSC_",
		"SW_",
		"FF_",
		"SYN_",
		"REP_",
		"INPUT_PROP_",
]

blacklist = [
		"EV_VERSION",
		"BTN_MISC",
		"BTN_MOUSE",
		"BTN_JOYSTICK",
		"BTN_GAMEPAD",
		"BTN_DIGI",
		"BTN_WHEEL",
		"BTN_TRIGGER_HAPPY"
]

btn_additional = [
		[0, "BTN_A"],
		[0, "BTN_B"],
		[0, "BTN_X"],
		[0, "BTN_Y"],
]

names = [
		"REL_",
		"ABS_",
		"KEY_",
		"BTN_",
		"LED_",
		"SND_",
		"MSC_",
		"SW_",
		"FF_",
		"SYN_",
		"REP_",
]

def convert(name):
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()

def print_enums(bits, prefix):
        if prefix == "ev":
                enum_name = "EventType"
        else:
                enum_name = prefix.upper()

	if  not hasattr(bits, prefix):
		return

	print("#[allow(non_camel_case_types)]")
	print("#[derive(Copy)]")
	print("#[derive(Clone)]")
	print("#[derive(PartialEq)]")
	print("pub enum %s {" % enum_name)
	for val, name in list(getattr(bits, prefix).items()):
		print("    %s = %s," % (name, val))
	if prefix == "key":
		for val, name in list(getattr(bits, "btn").items()):
			print("    %s = %s," % (name, val))
	print("}");
	print("");

def print_enums_convert_fn(bits, prefix):
        if prefix == "ev":
                fn_name = "EventType"
        else:
                fn_name = prefix.upper()

	if  not hasattr(bits, prefix):
		return

        print("pub fn %s(code: u32) -> Option<%s> {" %(convert(fn_name), fn_name))
        print("    match code {")
	for val, name in list(getattr(bits, prefix).items()):
                print("        %s => Some(%s::%s)," % (val, fn_name, name))
	if prefix == "key":
		for val, name in list(getattr(bits, "btn").items()):
                        print("        %s => Some(%s::%s)," % (val, fn_name, name))
        print("        _ => None")
	print("    }");
	print("}");
	print("");

def print_event_code(bits, prefix):
	if  not hasattr(bits, prefix):
		return

	print("#[allow(non_camel_case_types)]")
	print("#[derive(Copy)]")
	print("#[derive(Clone)]")
	print("#[derive(PartialEq)]")
	print("pub enum EventCode {")
	for val, name in list(getattr(bits, prefix).items()):
            if name[3:]+"_" in names:
                    print("    %s(%s)," % (name, name[3:]))
            elif name == "EV_FF_STATUS":
                    print("    EV_FF_STATUS(FF),")
            else:
                    print("    %s," % (name))
	if prefix == "key":
		for val, name in list(getattr(bits, "btn").items()):
			print("    %s = %s," % (name, val))
	print("}");
	print("");

def print_bits(bits, prefix):
	if  not hasattr(bits, prefix):
		return
	print("pub enum %s_map= {" % (prefix))
	for val, name in list(getattr(bits, prefix).items()):
		print("	[%s] = \"%s\"," % (name, name))
	if prefix == "key":
		for val, name in list(getattr(bits, "btn").items()):
			print("	[%s] = \"%s\"," % (name, name))
	print("};")
	print("")

def print_map(bits):
	print("pub enum event_type_map = {")

	for prefix in prefixes:
		if prefix == "BTN_" or prefix == "EV_" or prefix == "INPUT_PROP_":
			continue
		print("	[EV_%s] = %s_map," % (prefix[:-1], prefix[:-1].lower()))

	print("};")
	print("")

	print("pub enum ev_max = {")
	print("	[0 ... EV_MAX] = -1,")
	for prefix in prefixes:
		if prefix == "BTN_" or prefix == "EV_" or prefix == "INPUT_PROP_":
			continue
		print("	[EV_%s] = %s_MAX," % (prefix[:-1], prefix[:-1]))
	print("};")
	print("")

def print_lookup(bits, prefix):
	if not hasattr(bits, prefix):
		return

	names = list(getattr(bits, prefix).items())
	if prefix == "btn":
		names = names + btn_additional;

	for val, name in sorted(names, key=lambda e: e[1]):
		print("	{ .name = \"%s\", .value = %s }," % (name, name))

def print_lookup_table(bits):
	print("struct name_entry {")
	print("	const char *name;")
	print("	unsigned int value;")
	print("};")
	print("")
	print("static const struct name_entry ev_names[] = {")
	print_lookup(bits, "ev")
	print("};")
	print("")

	print("static const struct name_entry code_names[] = {")
	for prefix in sorted(names, key=lambda e: e):
		print_lookup(bits, prefix[:-1].lower())
	print("};")
	print("")
	print("static const struct name_entry prop_names[] = {")
	print_lookup(bits, "input_prop")
	print("};")
	print("")

def print_mapping_table(bits):
	for prefix in prefixes:
		if prefix == "BTN_":
			continue
                print_enums(bits, prefix[:-1].lower())
                print_enums_convert_fn(bits, prefix[:-1].lower())
                if prefix == "EV_":
                        print_event_code(bits, prefix[:-1].lower())
                

#	for prefix in prefixes:
#		if prefix == "BTN_":
#			continue
#		print_bits(bits, prefix[:-1].lower())
#
#	print_map(bits)
#	print_lookup_table(bits)
#
def parse_define(bits, line):
	m = re.match(r"^#define\s+(\w+)\s+(\w+)", line)
	if m == None:
		return

	name = m.group(1)

	if name in blacklist:
		return

	try:
		value = int(m.group(2), 0)
	except ValueError:
		return

	for prefix in prefixes:
		if not name.startswith(prefix):
			continue

		attrname = prefix[:-1].lower()

		if not hasattr(bits, attrname):
			setattr(bits, attrname, {})
		b = getattr(bits, attrname)
		b[value] = name

def parse(fp):
	bits = Bits()

	lines = fp.readlines()
	for line in lines:
		if not line.startswith("#define"):
			continue
		parse_define(bits, line)

	return bits

def usage(prog):
	print("Usage: %s /path/to/linux/input.h" % prog)

if __name__ == "__main__":
	if len(sys.argv) < 2:
		usage(sys.argv[0])
		sys.exit(2)

	print("/* THIS FILE IS GENERATED, DO NOT EDIT */")
	print("")

        if len(sys.argv) == 2:
                with open(sys.argv[1]) as f:
                        bits = parse(f)
                        print_mapping_table(bits)
                        sys.exit(2)

        for i in (1, len(sys.argv) - 1):
                with open(sys.argv[i]) as f:
                        bits = parse(f)
                        print_mapping_table(bits)
