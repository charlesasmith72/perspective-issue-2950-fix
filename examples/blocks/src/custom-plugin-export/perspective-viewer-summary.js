await customElements.whenDefined(
    "perspective-viewer-plugin",
);

const BasePlugin = customElements.get(
    "perspective-viewer-plugin",
);

class SummaryPlugin extends BasePlugin {
    get_static_config() {
        return {
            name: "Summary",
            select_mode: "select",
            priority: 1,
            initial: {
                count: 1,
                names: ["Sales"],
            },
        };
    }

    async draw(view) {
        const rowCount = await view.num_rows();
        const schema = await view.schema();

        this.innerHTML = `
            <style>
        .summary {
            box-sizing: border-box;
            height: 100%;
            padding: 40px;
            font-family: Arial, sans-serif;
        }

        .summary h1 {
            margin-top: 0;
        }
    </style>

            <div class="summary">
                <h1>Summary</h1>
        <p>The Summary plugin is a custom Perspective visualization that displays a simple overview of the current dataset instead of rendering a full table or chart. It shows key information such as the number of rows and the selected columns, providing a lightweight example of how third-party plugins can integrate with Perspective, participate in the viewer’s plugin system, and remain available after HTML export.</p>
                <p>
                    <strong>Rows:</strong>
                    ${rowCount}
                </p>

                <p>
                    <strong>Columns:</strong>
                    ${Object.keys(schema).join(", ")}
                </p>
            </div>
        `;
    }

    async update(view) {
        await this.draw(view);
    }
}

customElements.define(
    "perspective-viewer-summary",
    SummaryPlugin,
);