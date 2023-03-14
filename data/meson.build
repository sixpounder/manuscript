desktop_data = configuration_data()
desktop_data.set('APPLICATION_ID', application_id)

desktop_file = i18n.merge_file(
       input: configure_file(
         input: 'io.sixpounder.Manuscript.desktop.in.in',
         output: 'io.sixpounder.Manuscript.desktop.in',
         configuration: desktop_data,
       ),
       output: 'io.sixpounder.Manuscript.desktop',
         type: 'desktop',
       po_dir: '../po',
      install: true,
  install_dir: join_paths(get_option('datadir'), 'applications')
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

# appstream_file = i18n.merge_file(
#         input: 'io.sixpounder.Manuscript.appdata.xml.in',
#        output: 'io.sixpounder.Manuscript.appdata.xml',
#        po_dir: '../po',
#       install: true,
#   install_dir: join_paths(get_option('datadir'), 'appdata')
# )
appstream_data = configuration_data()
appstream_data.set('APPLICATION_ID', application_id)
appstream_file = i18n.merge_file(
  input: configure_file(
    input: 'io.sixpounder.Manuscript.appdata.xml.in.in',
    output: 'io.sixpounder.Manuscript.appdata.xml.in',
    configuration: appstream_data,
  ),
  output: 'io.sixpounder.Manuscript.appdata.xml',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'appdata')
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util, args: ['validate', appstream_file])
endif

install_data('io.sixpounder.Manuscript.gschema.xml',
  install_dir: join_paths(get_option('datadir'), 'glib-2.0/schemas')
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file',
       compile_schemas,
       args: ['--strict', '--dry-run', meson.current_source_dir()])
endif

install_data(
    join_paths('application-manuscript.xml'),
    install_dir: join_paths(manuscript_datadir, 'mime', 'packages')
)

subdir('icons')

message('Installing default font')
install_data(
    join_paths('fonts', 'iAWriterDuospace-Regular.otf'),
    install_dir: join_paths(get_option('datadir'), 'fonts')
)

install_data(
    join_paths('fonts', 'iAWriterDuospace-Bold.otf'),
    install_dir: join_paths(get_option('datadir'), 'fonts')
)

install_data(
    join_paths('fonts', 'iAWriterDuospace-BoldItalic.otf'),
    install_dir: join_paths(get_option('datadir'), 'fonts')
)

install_data(
    join_paths('fonts', 'iAWriterDuospace-Italic.otf'),
    install_dir: join_paths(get_option('datadir'), 'fonts')
)
message('Default font installed')