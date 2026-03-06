// ── Editing Adjustment State ──
// Type definitions and defaults for all editing parameters

export interface ToneCurvePoint {
    x: number; // 0-255
    y: number; // 0-255
}

export interface HSLChannelState {
    hue: number;        // -180 to 180
    saturation: number; // -100 to 100
    luminance: number;  // -100 to 100
}

export interface ColorWheelState {
    hue: number;        // 0-360
    saturation: number; // 0-100
    luminance: number;  // -100 to 100
}

export interface AdjustmentState {
    // Basic
    temperature: number;
    tint: number;
    exposure: number;
    contrast: number;
    highlights: number;
    shadows: number;
    whites: number;
    blacks: number;
    texture: number;
    clarity: number;
    dehaze: number;
    vibrance: number;
    saturation: number;

    // Tone Curve
    toneCurveChannel: 'rgb' | 'r' | 'g' | 'b';
    toneCurveRgb: ToneCurvePoint[];
    toneCurveR: ToneCurvePoint[];
    toneCurveG: ToneCurvePoint[];
    toneCurveB: ToneCurvePoint[];

    // HSL (8 color ranges)
    hsl: HSLChannelState[];

    // Color Grading
    cgShadows: ColorWheelState;
    cgMidtones: ColorWheelState;
    cgHighlights: ColorWheelState;
    cgBlending: number;
    cgBalance: number;

    // Detail
    sharpenAmount: number;
    sharpenRadius: number;
    sharpenDetail: number;
    sharpenMasking: number;
    nrLuminance: number;
    nrLumDetail: number;
    nrLumContrast: number;
    nrColor: number;
    nrColorDetail: number;
    nrColorSmooth: number;

    // Lens Corrections
    lensDistortion: number;
    lensVignetting: number;
    lensCaRed: number;
    lensCaBlue: number;

    // Effects
    vignetteAmount: number;
    vignetteMidpoint: number;
    vignetteRoundness: number;
    vignetteFeather: number;
    vignetteHighlights: number;
    grainAmount: number;
    grainSize: number;
    grainRoughness: number;

    // Calibration
    calShadowTint: number;
    calRedHue: number;
    calRedSat: number;
    calGreenHue: number;
    calGreenSat: number;
    calBlueHue: number;
    calBlueSat: number;
}

export const HSL_COLORS = ['Red', 'Orange', 'Yellow', 'Green', 'Aqua', 'Blue', 'Purple', 'Magenta'] as const;

export const HSL_COLOR_HEX: Record<string, string> = {
    Red: '#ef4444',
    Orange: '#f97316',
    Yellow: '#eab308',
    Green: '#22c55e',
    Aqua: '#06b6d4',
    Blue: '#3b82f6',
    Purple: '#8b5cf6',
    Magenta: '#d946ef',
};

function makeDefaultHSL(): HSLChannelState[] {
    return Array.from({ length: 8 }, () => ({ hue: 0, saturation: 0, luminance: 0 }));
}

function makeDefaultWheel(): ColorWheelState {
    return { hue: 0, saturation: 0, luminance: 0 };
}

export const defaultAdjustments: AdjustmentState = {
    temperature: 6500,
    tint: 0,
    exposure: 0,
    contrast: 0,
    highlights: 0,
    shadows: 0,
    whites: 0,
    blacks: 0,
    texture: 0,
    clarity: 0,
    dehaze: 0,
    vibrance: 0,
    saturation: 0,

    toneCurveChannel: 'rgb',
    toneCurveRgb: [{ x: 0, y: 0 }, { x: 255, y: 255 }],
    toneCurveR: [{ x: 0, y: 0 }, { x: 255, y: 255 }],
    toneCurveG: [{ x: 0, y: 0 }, { x: 255, y: 255 }],
    toneCurveB: [{ x: 0, y: 0 }, { x: 255, y: 255 }],

    hsl: makeDefaultHSL(),

    cgShadows: makeDefaultWheel(),
    cgMidtones: makeDefaultWheel(),
    cgHighlights: makeDefaultWheel(),
    cgBlending: 50,
    cgBalance: 0,

    sharpenAmount: 0,
    sharpenRadius: 1.0,
    sharpenDetail: 25,
    sharpenMasking: 0,
    nrLuminance: 0,
    nrLumDetail: 50,
    nrLumContrast: 50,
    nrColor: 0,
    nrColorDetail: 50,
    nrColorSmooth: 50,

    lensDistortion: 0,
    lensVignetting: 0,
    lensCaRed: 0,
    lensCaBlue: 0,

    vignetteAmount: 0,
    vignetteMidpoint: 50,
    vignetteRoundness: 0,
    vignetteFeather: 50,
    vignetteHighlights: 0,
    grainAmount: 0,
    grainSize: 25,
    grainRoughness: 50,

    calShadowTint: 0,
    calRedHue: 0,
    calRedSat: 0,
    calGreenHue: 0,
    calGreenSat: 0,
    calBlueHue: 0,
    calBlueSat: 0,
};

export function cloneAdjustments(adj: AdjustmentState): AdjustmentState {
    return {
        ...adj,
        toneCurveRgb: adj.toneCurveRgb.map(p => ({ ...p })),
        toneCurveR: adj.toneCurveR.map(p => ({ ...p })),
        toneCurveG: adj.toneCurveG.map(p => ({ ...p })),
        toneCurveB: adj.toneCurveB.map(p => ({ ...p })),
        hsl: adj.hsl.map(h => ({ ...h })),
        cgShadows: { ...adj.cgShadows },
        cgMidtones: { ...adj.cgMidtones },
        cgHighlights: { ...adj.cgHighlights },
    };
}

/** Convert AdjustmentState to the payload format expected by Rust */
export function toRustPayload(adj: AdjustmentState): Record<string, unknown> {
    return {
        temperature: adj.temperature,
        tint: adj.tint,
        exposure: adj.exposure,
        contrast: adj.contrast,
        highlights: adj.highlights,
        shadows: adj.shadows,
        whites: adj.whites,
        blacks: adj.blacks,
        texture: adj.texture,
        clarity: adj.clarity,
        dehaze: adj.dehaze,
        vibrance: adj.vibrance,
        saturation: adj.saturation,

        toneCurveRgb: adj.toneCurveRgb.map(p => [p.x, p.y]),
        toneCurveR: adj.toneCurveR.map(p => [p.x, p.y]),
        toneCurveG: adj.toneCurveG.map(p => [p.x, p.y]),
        toneCurveB: adj.toneCurveB.map(p => [p.x, p.y]),

        hslHue: adj.hsl.map(h => h.hue),
        hslSat: adj.hsl.map(h => h.saturation),
        hslLum: adj.hsl.map(h => h.luminance),

        cgShadowsHue: adj.cgShadows.hue,
        cgShadowsSat: adj.cgShadows.saturation,
        cgShadowsLum: adj.cgShadows.luminance,
        cgMidtonesHue: adj.cgMidtones.hue,
        cgMidtonesSat: adj.cgMidtones.saturation,
        cgMidtonesLum: adj.cgMidtones.luminance,
        cgHighlightsHue: adj.cgHighlights.hue,
        cgHighlightsSat: adj.cgHighlights.saturation,
        cgHighlightsLum: adj.cgHighlights.luminance,
        cgBlending: adj.cgBlending,
        cgBalance: adj.cgBalance,

        sharpenAmount: adj.sharpenAmount,
        sharpenRadius: adj.sharpenRadius,
        sharpenDetail: adj.sharpenDetail,
        sharpenMasking: adj.sharpenMasking,
        nrLuminance: adj.nrLuminance,
        nrLumDetail: adj.nrLumDetail,
        nrLumContrast: adj.nrLumContrast,
        nrColor: adj.nrColor,
        nrColorDetail: adj.nrColorDetail,
        nrColorSmooth: adj.nrColorSmooth,

        lensDistortion: adj.lensDistortion,
        lensVignetting: adj.lensVignetting,
        lensCaRed: adj.lensCaRed,
        lensCaBlue: adj.lensCaBlue,

        vignetteAmount: adj.vignetteAmount,
        vignetteMidpoint: adj.vignetteMidpoint,
        vignetteRoundness: adj.vignetteRoundness,
        vignetteFeather: adj.vignetteFeather,
        vignetteHighlights: adj.vignetteHighlights,
        grainAmount: adj.grainAmount,
        grainSize: adj.grainSize,
        grainRoughness: adj.grainRoughness,

        calShadowTint: adj.calShadowTint,
        calRedHue: adj.calRedHue,
        calRedSat: adj.calRedSat,
        calGreenHue: adj.calGreenHue,
        calGreenSat: adj.calGreenSat,
        calBlueHue: adj.calBlueHue,
        calBlueSat: adj.calBlueSat,
    };
}

// ── Filter Presets ──

export interface FilterPreset {
    id: string;
    name: string;
    adjustments: Partial<AdjustmentState>;
}

export const filterPresets: FilterPreset[] = [
    {
        id: 'auto', name: 'Auto',
        adjustments: { exposure: 0.3, contrast: 10, highlights: -15, shadows: 15, vibrance: 15 },
    },
    {
        id: 'vivid', name: 'Vivid',
        adjustments: { saturation: 25, contrast: 15, vibrance: 30 },
    },
    {
        id: 'matte', name: 'Matte / Faded',
        adjustments: { contrast: -15, blacks: 20, saturation: -10 },
    },
    {
        id: 'bw', name: 'Black & White',
        adjustments: { saturation: -100, contrast: 15 },
    },
    {
        id: 'cool-shadows', name: 'Cool Shadows',
        adjustments: { temperature: 5800, tint: -10, shadows: 15, highlights: -10 },
    },
    {
        id: 'golden-hour', name: 'Golden Hour',
        adjustments: { temperature: 7500, tint: 10, exposure: 0.2, contrast: 10, vibrance: 20 },
    },
    {
        id: 'cinematic', name: 'Teal & Orange',
        adjustments: { temperature: 5500, contrast: 20, saturation: -10, vibrance: 25 },
    },
];
