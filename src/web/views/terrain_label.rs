//! French names of terrains.

use crate::game_data::terrain::Terrain;

/// French name of a path terrain.
pub fn terrain_label(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::Any => "tout terrain",
        Terrain::Desolation => "désolation",
        Terrain::Forest => "forêt",
        Terrain::Hills => "collines",
        Terrain::Mountains => "montagnes",
        Terrain::Sun => "plaine",
    }
}
