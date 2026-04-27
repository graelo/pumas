// Pumas Dashboard Configuration
// Edit this file to customize the dashboard layout and behavior.
const DASHBOARD_CONFIG = {
    // How often to poll for new metrics (milliseconds)
    refreshInterval: 2000,

    // Default theme: 'light' | 'dark' | 'auto' (follows system preference)
    defaultTheme: 'auto',

    // Chart definitions.
    // row/col: position in a 12-column CSS grid.
    // width:  columns to span (1-12).
    // height: chart container height in pixels.
    charts: {
        cpuUtil: {
            show: true,
            row: 0, col: 0, width: 6, height: 280,
            title: 'CPU Utilization',
        },
        cpuFreq: {
            show: true,
            row: 0, col: 6, width: 6, height: 280,
            title: 'CPU Frequency',
        },
        gpu: {
            show: true,
            row: 1, col: 0, width: 6, height: 280,
            title: 'GPU',
        },
        power: {
            show: true,
            row: 1, col: 6, width: 6, height: 280,
            title: 'Power Consumption',
        },
        memory: {
            show: true,
            row: 2, col: 0, width: 12, height: 220,
            title: 'Memory',
        },
        thermal: {
            show: true,
            row: 3, col: 0, width: 12, height: 100,
            title: 'Thermal Pressure',
        },
    },
};
