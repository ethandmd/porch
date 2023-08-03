project('porch', 'c', 'cpp',
	default_options : [
		'werror=true',
		'warning_level=2',
		'cpp_std=c++17',
	])

src_files = files([
	'client.cpp',
    'event_loop.cpp',
])

# Point your PKG_CONFIG_PATH environment variable to the
# libcamera install path libcamera.pc file ($prefix/lib/pkgconfig/libcamera.pc)
deps = [
      dependency('libcamera', required : true),
      dependency('libevent_pthreads'),
]

cpp_arguments = [ '-Wno-unused-parameter', ]

add_project_arguments(cpp_arguments, language : 'cpp')

# simple-cam executable
porch = executable('porch', src_files,
                        dependencies : deps)
