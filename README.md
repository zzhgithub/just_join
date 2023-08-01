# just_join



# 性能优化方法
https://ui.perfetto.dev/
https://github.com/bevyengine/bevy/blob/main/docs/profiling.md



# 材质配置工具

读取某个特定的文件夹。
然后把所有的贴图 都展示出来。
然后可以选择  正方体的六个面使用什么样的数据

在点击保存时生成配置文件
在游戏运行时可以直接根据配置文件加载数据。

需要egui 还有对cube的贴图进行实时预览

## 配置文件的设计
```json
{
    voxels:{
        // 0 是voxel类型的number
        0: {
            default: {index:0,path:},// 这里是找不到时取默认的贴图索引
            'normal_number':{index:0,path:},// 这是特定法向量索引配置
        }
        ...
    },
    // 这是文件列表
    files:[
        'path/a',
        'path/b',
        ...
    ]
}
```
解释：
1. voxels 来通过 voxel类型和 法向量获取到要使用的图片索引
2. files是在 加载blessMaterail时的图片贴图列表。这样 可以在mesh传递时使用 在data内直接去索引
3. 这使用的数据 是ron格式 而不是使用的json

# 材质加载
todo: 要修改一个可以查看 并且修改数据的简单单元。