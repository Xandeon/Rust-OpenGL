

use beryllium::*;
use ogl33::*;


#[allow(non_snake_case)]


// NDC, normalized display coordinate data
type Vertex = [f32; 3];
const VERTICES: [Vertex; 3] = 
    [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]]; // groups of 3 is a triangle

// create a window
const WINDOW_TITLE: &str = "Triangle: Draw Arrays";

// vertex shader program, written in GLSL, (could write this in separate file and load in)
const VERT_SHADER: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 pos;
    void main() {
        gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    }"#;

const FRAG_SHADER: &str = r#"
    #version 330 core
    out vec4 final_color;
    void main() {
        final_color = vec4(1.0, 0.5, 0.2, 1.0);
    }"#;


fn main(){

    println!("initializing SDL");
    let sdl = beryllium::Sdl::init(init::InitFlags::EVERYTHING);

    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_gl_profile(video::GlProfile::Core).unwrap();


    let win_args = video::CreateWinArgs {
        title: WINDOW_TITLE,
        width: 800,
        height: 600,
        allow_high_dpi: true,
        borderless: false,
        resizable: true,
    };

    let win = sdl
        .create_gl_window(win_args)
        .expect("couldn't make a window and context");

    win.set_swap_interval(video::GlSwapInterval::Vsync).unwrap();

    unsafe {

        // load OpenGL functions
        load_gl_with(|f_name: *const i8| win.get_proc_address(f_name as *const u8));    //ogl33 crate

        // set background color
        glClearColor(0.2, 0.2, 0.2, 1.0); // set bg to nice color          ogl33 crate


        // ----------------- part 1: Send Data to GPU -=-=-=-=-=-=-=-=-=-=-==--=-==-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=

        // set up vertex array object
        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);

        glBindVertexArray(vao); // bind the array

        // setup vertex buffer object
        let mut vbo = 0;
        glGenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);

        // bind the vertex buffer to a target, context wide GL functions act on vbo
        // api handles things ona  context wide mode where the bound vbo is where functions act.
        // the user doesn't manage this but it is confusing
        glBindBuffer(GL_ARRAY_BUFFER, vbo);

        // buffer the data -----------------------------------------------------------------------------------------------
        glBufferData(
            GL_ARRAY_BUFFER,          
            size_of_val(&VERTICES) as isize,    // size of the object (vertices, which is an array of vertexes)
            VERTICES.as_ptr().cast(),   // technically a void pointer, but doesnt exist in rust
            GL_STATIC_DRAW,
        );

        // describe/enable  the vertex attribute pointer  -----------------------------------------------------------------
        glVertexAttribPointer(
            0,                      // index of attribute being described
            3,                       // number of components in attribute
            GL_FLOAT,               // type of associated data points
            GL_FALSE,
            /*The stride is the number of bytes from the start of this attribute in one vertex 
            to the start of the same attribute in the next vertex. 
            Since we have only one attribute right now, that's just size_of::<f32>() * 3.
            Alternately, we can use size_of::<Vertex>() and when we edit our type alias at the top later 
            on this vertex attribute value will automatically be updated for us. */
            size_of::<Vertex>().try_into().unwrap(), // tryinto is to get from usize to isize
            0 as *const _,              // confused af
        );

        glEnableVertexAttribArray(0);   // index of attribute bo enabled 


        // part 2: -------------------------- send program to GPU -=-=-=-=-=-==-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
        // program is for GPU to know what to do with bytes, vertexes, (groups of 3 f32 values)
        

        // create vertex shader, every GL program needs a vertex shader and fragment shader
        // shader is GPU lingo for program
        let vertex_shader = glCreateShader(GL_VERTEX_SHADER);
        assert_ne!(vertex_shader, 0); 

        glShaderSource(
            vertex_shader, 
            1,                                           
            &(VERT_SHADER.as_bytes().as_ptr().cast()), //send the GLSL code
            &(VERT_SHADER.len().try_into().unwrap()),
        );
        glCompileShader(vertex_shader); 
        // check if this compiled or not since it is technically dyamically compiling the GLSL code

        let mut success = 0;
        glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);


        println!("success: {}", success);

        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);  // room to ger log back
            let mut log_len = 0_i32;
            glGetShaderInfoLog( // get the shit
              vertex_shader,
              1024,
              &mut log_len,
              v.as_mut_ptr().cast(),
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }

        // same shit for frag shader

        let fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        glShaderSource(
            fragment_shader,
            1,
            &(FRAG_SHADER.as_bytes().as_ptr().cast()),
            &(FRAG_SHADER.len().try_into().unwrap()),
        );
        glCompileShader(fragment_shader);

        let mut success = 0;
        glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);  // i-v int-vector ?
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetShaderInfoLog(
            fragment_shader,
            1024,
            &mut log_len,
            v.as_mut_ptr().cast(),
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }
        
        // now we creat a program with the compiled shaders --------------------------------------------

        let shader_program = glCreateProgram();
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glLinkProgram(shader_program);

        //error checka again, seems like this could be functionalized
        glGetProgramiv(shader_program, GL_LINK_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetProgramInfoLog(
            shader_program,
            1024,
            &mut log_len,
            v.as_mut_ptr().cast(),
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Program Link Error: {}", String::from_utf8_lossy(&v));
        }

        // doesnt delete the program but detaches the thing
        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);

        // tell GL to use the program created
        glUseProgram(shader_program);

    }

    // start the main loop
    'main_loop: loop {
        // handle events this frame
        while let Some(event) = sdl.poll_events() {
            match event {
                (events::Event::Quit, _) => break 'main_loop,
                _ => (),
            }
        }
        // now the events are clear


        // here's where we could change the world state and draw.


        // draw 
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            glDrawArrays(GL_TRIANGLES, 0, 3); // draw the triangle   
        }

        win.swap_window(); // this updates the window
    }

}
