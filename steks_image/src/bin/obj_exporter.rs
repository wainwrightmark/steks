use std::fmt::Display;

use base64::Engine;
use bevy::{prelude::*, utils::HashMap};
pub use steks_common::prelude::*;

const SCALE: f32 = 1.0 / 50.0;

pub fn main() {
    println!("Let's go");

    let materials: Vec<Material> = ALL_SHAPES
        .iter()
        .map(|shape| Material::new(shape))
        .collect();

    let mat_file = MtlFile { materials };
    let material_file  = Some("steks_mat.mtl".to_string());

    let mat_file_path = format!("obj_exports/steks_mat.mtl");
    std::fs::write(mat_file_path, mat_file.to_string()).unwrap();

    let records = get_records();

    let mut all_groups: Vec<Group> = vec![];

    for (number, level) in CAMPAIGN_LEVELS.iter().enumerate() {
        let sv = ShapesVec::from(level);
        let hash = sv.hash();

        let Some(record) = records.get(&hash) else {
            continue;
        };

        let title: String = level
            .title
            .clone()
            .unwrap()
            .chars()
            .filter(|x| x.is_ascii_alphabetic())
            .collect();
        let number = number + 1;

        println!("{title}",);
        let mut offset: usize = 1;

        let shapes = ShapesVec::from_bytes(&record.image_blob);
        let groups: Vec<Object> = shapes
            .iter()
            .filter(|shape| !shape.state.is_void())
            .enumerate()
            .map(|(index, shape)| {
                let obj = Object::new(index, shape, offset);
                offset += obj.vertices.len();
                obj
            })
            .collect();
        let group: Group = Group {
            objects: groups,
            name: title.clone(),
        };

        all_groups.push(group.clone());
        let obj_file: ObjFile = ObjFile {
            material_file: material_file.clone() ,
            groups: vec![group],
        };
        let obj_file_path = format!("obj_exports/{number}_{title}.obj",);
        std::fs::write(obj_file_path, obj_file.to_string()).unwrap();
    }

    let mut offset = 0;
    for (index,  group) in all_groups.iter_mut().enumerate(){
        group.offset_vertices(offset);
        offset += group.count_vertices();

        let x = index % 6;
        let z = index / 6;

        let vector = Vec3::new(x as f32 * 50.0, 0.0, z as f32 * 50.0);


        group.offset_position(vector);
    }

    let path = format!("obj_exports/all.obj",);
    std::fs::write(path, ObjFile{groups: all_groups, material_file: material_file}.to_string()).unwrap();


}

pub struct MtlFile {
    pub materials: Vec<Material>,
}

impl Display for MtlFile{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for m in self.materials.iter(){
            m.fmt(f)?;
        }

        Ok(())
    }
}

pub struct Material {
    pub name: String,
    pub ambient_color: Color,
    pub diffuse_color: Color,
    pub specular_color: Color,
    /// between 0 and 1000.0
    pub specular_exponent: f32,
    /// transparency - 1.0 is opaque, 0.0 is transparent
    pub dissolve: f32,
    /// index of refraction
    pub optical_density: f32,
}

impl Display for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "newmtl {}", self.name)?;
        {
            let [r, g, b, _a] = self.ambient_color.as_rgba_f32();
            writeln!(f, "Ka {r} {g} {b}")?;
        }
        {
            let [r, g, b, _a] = self.diffuse_color.as_rgba_f32();
            writeln!(f, "Kd {r} {g} {b}")?;
        }
        {
            let [r, g, b, _a] = self.specular_color.as_rgba_f32();
            writeln!(f, "Ks {r} {g} {b}")?;
        }
        writeln!(f, "Ns {}", self.specular_exponent)?;
        writeln!(f, "Ni {}", self.optical_density)?;
        writeln!(f, "d {}", self.dissolve)?;

        Ok(())
    }
}

impl Material {
    pub fn new(shape: &GameShape) -> Self {
        let name = format!("{shape_name}_mat", shape_name = shape.name);
        let ambient_color = shape.default_fill_color(false);
        let diffuse_color = ambient_color * 1.0;
        let specular_color = ambient_color * 0.5;


        Self {
            name,
            ambient_color,
            diffuse_color,
            specular_color,
            specular_exponent: 250.0,
            dissolve : 1.0,
            optical_density: 1.45,
        }
    }
}

pub struct ObjFile {
    pub material_file: Option<String>,
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone)]
pub struct Face(Vec<usize>);

#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub objects: Vec<Object>,
}

impl Group {
    pub fn offset_vertices(&mut self, offset: usize){
        for object in self.objects.iter_mut(){
            object.offset_vertices(offset);
        }
    }

    pub fn count_vertices(&self)-> usize{
        self.objects.iter().map(|x|x.vertices.len()).sum()
    }

    pub fn offset_position(&mut self, vector: Vec3){
        for obj in self.objects.iter_mut(){
            obj.offset_position(vector);
        }
    }
}

impl Object {
    pub fn offset_vertices(&mut self, offset: usize){
        for face in self.faces.iter_mut(){
            face.offset_vertices(offset);
        }
    }

    pub fn offset_position(&mut self, vector: Vec3){
        for v in self.vertices.iter_mut(){
            *v = *v + vector;
        }
    }
}

impl Face {
    pub fn offset_vertices(&mut self, offset: usize){
        for x in self.0.iter_mut(){
            *x = *x + offset
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub material: String,
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Face>,
}

impl Object {
    pub fn new(index: usize, shape: &EncodableShape, offset: usize) -> Self {
        let fatness: f32 = SHAPE_SIZE * SCALE * 0.25;
        let shape_name = shape.shape.game_shape().name;
        let name = if shape.state.is_locked() || shape.state.is_fixed() {format!("{shape_name}_{index}_locked",)} else{format!("{shape_name}_{index}",)};
        let material = format!("{shape_name}_mat");

        let vertices = shape
            .shape
            .game_shape()
            .body
            .get_vertices(SHAPE_SIZE * SCALE);
        let vertices_count_2d = vertices.len();
        let angle: Vec2 = Vec2::from_angle(shape.location.angle);

        let vertices: Vec<Vec3> = vertices
            .into_iter()
            .flat_map(|v| {
                let adjusted = v.rotate(angle) + (shape.location.position * SCALE);
                [
                    adjusted.extend(fatness * 1.0),
                    adjusted.extend(fatness * -1.0),
                ]
            })
            .collect();

        let mut faces = vec![];
        let mut front_face: Vec<usize> = vec![];
        let mut back_face: Vec<usize> = vec![];
        for index in 0..vertices_count_2d {
            let index = index * 2;
            front_face.push(index + offset);
            back_face.push(index + offset + 1);

            let new_face = vec![
                index + offset,
                index + 1 + offset,
                ((index + 3) % vertices.len()) + offset,
                ((index + 2) % vertices.len()) + offset,
            ];
            faces.push(Face(new_face));
        }
        faces.push(Face(front_face));
        faces.push(Face(back_face));

        Self {
            name,
            vertices,
            faces,
            material,
        }
    }
}

impl Display for ObjFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(mat_file) = &self.material_file {
            f.write_fmt(format_args!("mtllib {mat_file}\n"))?;
        }

        for group in self.groups.iter() {
            group.fmt(f)?;
            f.write_str("\n\n")?;
        }

        Ok(())
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("g {}\n", self.name))?;
        for object in self.objects.iter() {
            object.fmt(f)?;
        }
        Ok(())
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("o {}\n", self.name))?;

        for vertex in self.vertices.iter() {
            writeln!(f, "v {} {} {}\n", vertex.x, vertex.y, vertex.z)?;
        }

        writeln!(f, "usemtl {}", self.material)?;

        for face in self.faces.iter() {
            f.write_str("f")?;
            for x in face.0.iter() {
                f.write_str(" ")?;
                x.fmt(f)?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

fn get_records() -> HashMap<u64, Record> {
    let mut map: HashMap<u64, Record> = Default::default();
    for line in RECORDS_DATA.lines() {
        let mut split = line.split_ascii_whitespace();
        let hash = split.next().unwrap();
        let height = split.next().unwrap();
        let image_blob: &str = split.next().unwrap();

        let hash: u64 = hash.parse().unwrap();
        let height: f32 = height.parse().unwrap();
        let image_blob: Vec<u8> = base64::engine::general_purpose::URL_SAFE
            .decode(image_blob)
            .unwrap();

        let record: Record = Record {
            hash,
            height,
            image_blob,
        };

        map.insert(hash, record);
    }
    map
}

#[allow(dead_code)]
#[derive(Debug)]
struct Record {
    hash: u64,
    height: f32,
    image_blob: Vec<u8>,
}

const RECORDS_DATA: &'static str = include_str!("records.tsv");

//script to add rounded edges
/*
import bpy

# Select all objects in the scene
bpy.ops.object.select_all(action='SELECT')

# Go through each selected object and add a Bevel modifier
for obj in bpy.context.selected_objects:
    if obj.type == 'MESH':
        # Add a Bevel modifier
        bevel_mod = obj.modifiers.new(name="Bevel", type='BEVEL')

        # Adjust Bevel modifier settings as needed
        bevel_mod.width = 0.1  # Set the bevel width (adjust as desired)
        bevel_mod.segments = 8  # Set the number of segments (adjust as desired)
        bevel_mod.limit_method = 'ANGLE'  # Set the limit method

        bevel_mod.affect = 'VERTICES'

        # Apply the modifier to the object
        bpy.ops.object.modifier_apply(modifier="Bevel")

# Deselect all objects
bpy.ops.object.select_all(action='DESELECT')
 */
//script to add rigid bodies
 /*
 import bpy

# Deselect all objects
bpy.ops.object.select_all(action='DESELECT')

# Loop through all objects in the scene
for obj in bpy.context.scene.objects:
    if obj.type == 'MESH':
        if "locked" in obj.name.lower():
            # Create a passive rigid body
            obj.select_set(True)
            bpy.context.view_layer.objects.active = obj
            bpy.ops.rigidbody.object_add()
            bpy.context.object.rigid_body.type = 'PASSIVE'
        else:
            # Create an active rigid body
            obj.select_set(True)
            bpy.context.view_layer.objects.active = obj
            bpy.ops.rigidbody.object_add()
            bpy.context.object.rigid_body.type = 'ACTIVE'

# Deselect all objects again
bpy.ops.object.select_all(action='DESELECT')

  */