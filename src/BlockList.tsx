import Block from "./Block";

interface BlockListProps {
    blocks: any
}

const BlockList: React.FC<BlockListProps> = ({ blocks }) => {

    return (
        <div>
            {
                blocks &&
                blocks.map((block: any) => {
                    return <Block key={block.id} block={block} id={block.id} raw_data={block.raw_data} />
                })
            }
        </div>
    )
}

export default BlockList;