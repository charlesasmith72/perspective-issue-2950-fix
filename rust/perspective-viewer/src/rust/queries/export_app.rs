// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use itertools::Itertools;

static VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum ExportPlugin {
    Package {
        tag_name: String,
        package: String,
    },
    Module {
        tag_name: String,
        module: String,
    },
}

fn render_plugin(plugin: &ExportPlugin) -> String {
    match plugin {
        ExportPlugin::Package {
            tag_name,
            package,
        } => {
            format!(
                "import \"https://cdn.jsdelivr.net/npm/@perspective-dev/{package}@{VERSION}/dist/cdn/{tag_name}.js\";\n"
            )
        }
        ExportPlugin::Module {
            tag_name,
            module,
        } => {
            format!(
                "import {module:?};\n\
                await customElements.get(\"perspective-viewer\").registerPlugin(\n\
                {tag_name:?},\n\
                {module:?},\n\
                );\n"
            )
        }
    }
}

pub fn render(
    data: &str,
    layout: &str,
    plugins: &[ExportPlugin],
) -> String {
    let stmts = plugins.iter().map(render_plugin);
    let imports = Itertools::intersperse(stmts, " ".to_owned()).collect::<String>();

    format!("
<!DOCTYPE html>
<html lang=\"en\">
<head>
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1,maximum-scale=1,minimum-scale=1,user-scalable=no\"/>
<link rel=\"stylesheet\" crossorigin=\"anonymous\" href=\"https://cdn.jsdelivr.net/npm/@perspective-dev/viewer@{VERSION}/dist/css/themes.css\"/>
<script type=\"module\">
import perspective from \"https://cdn.jsdelivr.net/npm/@perspective-dev/client@{VERSION}/dist/cdn/perspective.js\";
import \"https://cdn.jsdelivr.net/npm/@perspective-dev/viewer@{VERSION}/dist/cdn/perspective-viewer.js\";
{imports}
const worker = await perspective.worker();
const binary_string = window.atob(window.data.textContent);
const len = binary_string.length;
const bytes = new Uint8Array(len);
for (let i = 0; i < len; i++) {{
bytes[i] = binary_string.charCodeAt(i);
}}
const layout = JSON.parse(window.layout.textContent);

await worker.table(bytes.buffer, {{
    name: layout.table,
}});

await window.viewer.load(worker);
await window.viewer.restore(layout);
</script>
<style>perspective-viewer{{position:absolute;top:0;left:0;right:0;bottom:0}}</style>
</head>
<body>
<script id='data' type=\"application/octet-stream\">{data}</script>
<script id='layout' type=\"application/json\">{layout}</script>
<perspective-viewer id='viewer'></perspective-viewer>
</body>
</html>
")
}
