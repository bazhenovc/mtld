# mtld
Material downloading and packing tool.

Supported features:
* Downloading material files from (Ambient CG)[https://ambientcg.com/] with specified resolution
* Resinging and packing material properties into different texture channels
* Compressing material textures with [Basis Universal](https://github.com/BinomialLLC/basis_universal)
* Generating Unity3D material files with all needed parameters (sRGB for Albedo, Linear for other textures; correct opacity settings, etc)

## Usage

    mtld.exe [FLAGS] [OPTIONS]

## Flags

        --basisu                    Compresses packed files with Basis Universal
        --basisu-single-threaded    Dont't use multithreading for compressing with Basis Universal
        --download                  Downloads .zip files
        --force-basisu              Force compressing with Basis Universal even when files exist
        --force-download            Forces download even when .zip files exist in the cache
        --force-pack                Force packing even when files exist
        --force-unity               Force generate Unity3D meta files even when files exist
    -h, --help                      Prints help information
        --pack                      Packs downloaded files
        --pack-single-threaded      Don't use multi threading for packing
        --unity                     Generates Unity3D meta files
    -V, --version                   Prints version information

## Options
        --basisu-cache-path <basisu-cache-path>
            Folder where compressed Basis Universal files will be stored [default: .mtld-basisu-cache]

        --download-cache <download-cache-path>
            Folder where downloaded .zip files will be stored [default: .mtld-download-cache]

        --download-extensions <download-extensions>
            Download using the first extension form this list, try next one if failed [default: JPG,PNG]

        --download-resolutions <download-resolutions>
            Download using the first resolution from this list, try next one if failed [default: 4K,3K,2K,1K]

        --download-type <download-type>
            Specifies types of assets to be downloaded [default: PhotoTexturePBR,DecalPBR,AtlasPBR]

        --pack-cache-path <pack-cache-path>
            Folder where packed files will be stored [default: .mtld-pack-cache]

        --pack-normal-map-type <pack-normal-map-type>          Normal map type to use for packing [default: OpenGL]
        --pack-target-height <pack-target-height>              Packed image height [default: 1024]
        --pack-target-width <pack-target-width>                Packed image width [default: 1024]
        --unity-cache-path <unity-cache-path>
            Folder where Unity3D meta files will be stored [default: .mtld-unity-cache]

        --unity-material-template <unity-material-template>
            Template file to generate Unity3D material meta files [default: templates/unity_material.template]

        --unity-texture-template <unity-texture-template>
            Template file to generate Unity3D texture meta files [default: templates/unity_texture.template]

        --user-agent <user-agent>
            Override default User-Agent header when making HTTP requests [default: Mozilla/5.0 (X11; Linux x86_64)
            AppleWebKit/537.36 (KHTML, like Gecko) Chrome/44.0.2403.157 Safari/537.36]
